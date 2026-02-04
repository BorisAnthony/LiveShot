use std::time::{Duration, Instant};
use anyhow::Result;
use headless_chrome::Tab;

/// Poll a JS expression that returns a boolean until it yields the expected value, or deadline.
fn poll_js(tab: &Tab, js: &str, expect: bool, deadline: Instant) -> Option<()> {
    while Instant::now() < deadline {
        let val = tab.evaluate(js, false)
            .ok()
            .and_then(|r| r.value)
            .and_then(|v| v.as_bool())
            .unwrap_or(!expect);
        if val == expect {
            return Some(());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    None
}

/// Run all YouTube-specific preparation after navigation:
/// dismiss consent, wait for ads, wait for video, theater mode, hide controls.
pub fn prepare(tab: &Tab, deadline: Instant, timeout_secs: u64) -> Result<()> {
    // Dismiss GDPR consent dialog (best-effort)
    for sel in [
        "button[aria-label*='Accept']",
        "ytd-consent-bump-v2-lightbox button[aria-label*='Accept']",
        "tp-yt-paper-dialog .buttons button",
    ] {
        let js = format!(
            r#"(function(){{ var b=document.querySelector('{}'); if(b){{ b.click(); return true; }} return false; }})()"#,
            sel
        );
        let clicked = tab.evaluate(&js, false)
            .ok()
            .and_then(|r| r.value)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if clicked {
            std::thread::sleep(Duration::from_secs(1));
            break;
        }
    }

    // Wait for ads to finish
    let ad_js = r#"(function(){
        var p=document.getElementById('movie_player');
        return p ? p.classList.contains('ad-showing') : false;
    })()"#;
    poll_js(tab, ad_js, false, deadline)
        .ok_or_else(|| anyhow::anyhow!("Timed out waiting for YouTube ads to finish ({}s)", timeout_secs))?;

    // Wait for <video> element to exist
    poll_js(tab, "document.querySelector('video') !== null", true, deadline)
        .ok_or_else(|| anyhow::anyhow!("Timed out after {}s waiting for a <video> element", timeout_secs))?;

    // Try to start playback programmatically
    let _ = tab.evaluate(
        r#"(function(){
            var v=document.querySelector('video');
            if(v && v.paused){ v.muted=true; v.play().catch(function(){}); }
            var p=document.getElementById('movie_player');
            if(p && typeof p.playVideo==='function') p.playVideo();
        })()"#,
        false,
    );

    // Wait for video to actually play
    let playing_js = r#"(function(){
        var v=document.querySelector('video');
        return v && v.readyState>=3 && !v.paused;
    })()"#;
    if poll_js(tab, playing_js, true, deadline).is_none() {
        let diag = tab.evaluate(
            r#"(function(){
                var v=document.querySelector('video');
                if(!v) return 'no video element';
                return 'readyState='+v.readyState+' paused='+v.paused+' src='+(v.src||v.currentSrc||'none');
            })()"#,
            false,
        )
        .ok()
        .and_then(|r| r.value)
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "unknown".to_string());
        anyhow::bail!("Timed out after {}s waiting for video to play. State: {}", timeout_secs, diag);
    }
    std::thread::sleep(Duration::from_millis(500)); // frame settle

    // Theater mode + hide controls
    let _ = tab.evaluate(
        r#"(function(){
            var btn=document.querySelector('.ytp-size-button');
            if(btn) btn.click();
        })()"#,
        false,
    );
    std::thread::sleep(Duration::from_millis(500));

    let _ = tab.evaluate(
        r#"(function(){
            var p=document.getElementById('movie_player');
            if(p) p.dispatchEvent(new MouseEvent('mouseleave',{bubbles:true}));
            document.body.dispatchEvent(new MouseEvent('mousemove',{clientX:0,clientY:0,bubbles:true}));
        })()"#,
        false,
    );
    std::thread::sleep(Duration::from_secs(3)); // controls fade-out

    Ok(())
}
