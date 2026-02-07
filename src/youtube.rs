use std::time::{Duration, Instant};
use anyhow::Result;
use headless_chrome::Tab;
use headless_chrome::protocol::cdp::Network;

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

/// Set YouTube/Google consent cookies via CDP so the GDPR dialog never appears.
/// Must be called *before* navigating to the YouTube URL.
pub fn set_consent_cookie(tab: &Tab) -> Result<()> {
    let expiry = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        + 365.25 * 24.0 * 60.0 * 60.0;

    let make_cookie = |domain: &str, url: &str| Network::CookieParam {
        name: "SOCS".to_string(),
        value: "CAISHAgCEhJnd3NfMjAyNDAxMTAtMF9SQzIaAmVuIAEaBgiA_LyaBg".to_string(),
        url: Some(url.to_string()),
        domain: Some(domain.to_string()),
        path: Some("/".to_string()),
        secure: Some(true),
        http_only: None,
        same_site: None,
        expires: Some(expiry),
        priority: None,
        same_party: None,
        source_scheme: None,
        source_port: None,
        partition_key: None,
    };

    tab.call_method(Network::SetCookies {
        cookies: vec![
            make_cookie(".youtube.com", "https://www.youtube.com"),
            make_cookie(".google.com", "https://www.google.com"),
        ],
    })?;

    Ok(())
}

/// Check whether the GDPR consent dialog is blocking the page.
fn has_consent_dialog(tab: &Tab) -> bool {
    let js = r#"(function(){
        if (window.location.href.indexOf('consent') !== -1) return true;
        if (document.querySelector('ytd-consent-bump-v2-lightbox')) return true;
        if (document.querySelector('tp-yt-paper-dialog')) return true;
        var btns = document.querySelectorAll('button');
        for (var i = 0; i < btns.length; i++) {
            var t = btns[i].textContent.trim();
            if (t === 'Reject all' || t === 'Accept all') return true;
        }
        return false;
    })()"#;

    tab.evaluate(js, false)
        .ok()
        .and_then(|r| r.value)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Dismiss the GDPR consent dialog by setting cookies and re-navigating.
/// This avoids fragile DOM clicks — we just set the consent cookie from
/// the YouTube origin and load the page again.
fn dismiss_consent(tab: &Tab, url: &str) -> Result<()> {
    if !has_consent_dialog(tab) {
        return Ok(());
    }

    let current = tab.get_url();

    // If we got redirected off YouTube entirely (e.g. consent.google.com),
    // navigate to youtube.com first so document.cookie scopes correctly.
    if !current.contains("youtube.com") {
        tab.navigate_to("https://www.youtube.com")?
            .wait_until_navigated()?;
    }

    // Set consent cookies via document.cookie on the YouTube origin.
    tab.evaluate(
        r#"(function(){
            var d = ';domain=.youtube.com;path=/;secure;max-age=31536000';
            document.cookie = 'SOCS=CAISHAgCEhJnd3NfMjAyNDAxMTAtMF9SQzIaAmVuIAEaBgiA_LyaBg' + d;
            document.cookie = 'CONSENT=YES+cb.20210420-17-p0.en+FX+920' + d;
        })()"#,
        false,
    )?;

    // Re-navigate to the target URL — the consent cookie is now set,
    // so YouTube should skip the GDPR dialog.
    tab.navigate_to(url)?
        .wait_until_navigated()?;

    Ok(())
}

/// Run all YouTube-specific preparation after navigation:
/// dismiss consent, wait for ads, wait for video, theater mode, hide controls.
pub fn prepare(tab: &Tab, deadline: Instant, timeout_secs: u64, url: &str) -> Result<()> {
    dismiss_consent(tab, url)?;

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
