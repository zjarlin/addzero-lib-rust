//! Browser profile data used to keep automated sessions internally consistent.
//!
//! This module models common browser/device combinations for authorized testing.
//! The [`FingerprintProfile::inject`] method applies CDP emulation settings and
//! injects a consistency script that covers navigator values, screen metrics,
//! WebGL, canvas noise, audio context, plugin spoofing, and webdriver property
//! removal.

use crate::{BrowserAutomationError, BrowserAutomationResult};
use az_derive_aliases::{apply, plain_copy_eq, serde_eq};
use headless_chrome::Tab;
use headless_chrome::protocol::cdp::{Emulation, Page};
use rand::Rng;

/// Browser and device identity values for one automation session.
#[apply(serde_eq)]
pub struct FingerprintProfile {
    /// User agent string applied through CDP.
    pub user_agent: String,
    /// Browser viewport width and height.
    pub viewport: (u32, u32),
    /// Preferred browser languages in priority order.
    pub languages: Vec<String>,
    /// Navigator platform value, such as `Win32`, `MacIntel`, or `Linux x86_64`.
    pub platform: String,
    /// Informational WebGL vendor associated with the profile.
    pub webgl_vendor: String,
    /// Informational WebGL renderer associated with the profile.
    pub webgl_renderer: String,
    /// Screen width and height associated with the viewport.
    pub screen_resolution: (u32, u32),
    /// Screen color depth in bits.
    pub color_depth: u32,
    /// IANA timezone identifier used for CDP timezone emulation.
    pub timezone: String,
    /// Hardware concurrency value associated with the profile.
    pub hardware_concurrency: u8,
    /// Device memory value associated with the profile.
    pub device_memory: u8,
}

impl FingerprintProfile {
    /// Picks a realistic profile from [`SELECTION_POOL`].
    #[must_use]
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..SELECTION_POOL.len());
        SELECTION_POOL[index].to_profile()
    }

    /// Picks a realistic profile matching a platform family.
    ///
    /// Accepted platform values include exact navigator platforms such as
    /// `Win32`, `MacIntel`, and `Linux x86_64`, plus family aliases such as
    /// `windows`, `macos`, and `linux`. Unknown values fall back to
    /// [`FingerprintProfile::random`].
    #[must_use]
    pub fn for_platform(platform: &str) -> Self {
        let matches = SELECTION_POOL
            .iter()
            .filter(|profile| platform_matches(platform, profile.platform))
            .collect::<Vec<_>>();

        if matches.is_empty() {
            return Self::random();
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..matches.len());
        matches[index].to_profile()
    }

    /// Applies profile values to a tab through CDP before navigation.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError::Browser`] if CDP rejects the user
    /// agent, viewport, timezone, or initialization script.
    pub fn inject(&self, tab: &Tab) -> BrowserAutomationResult<()> {
        let accept_language = self.languages.join(",");
        tab.set_user_agent(
            &self.user_agent,
            Some(&accept_language),
            Some(&self.platform),
        )
        .map_err(to_browser_error)?;

        tab.call_method(Emulation::SetTimezoneOverride {
            timezone_id: self.timezone.clone(),
        })
        .map_err(to_browser_error)?;

        tab.call_method(Emulation::SetDeviceMetricsOverride {
            width: self.viewport.0,
            height: self.viewport.1,
            device_scale_factor: 1.0,
            mobile: false,
            scale: None,
            screen_width: Some(self.screen_resolution.0),
            screen_height: Some(self.screen_resolution.1),
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
            display_feature: None,
            device_posture: None,
        })
        .map_err(to_browser_error)?;

        tab.call_method(Page::AddScriptToEvaluateOnNewDocument {
            source: self.navigator_consistency_script()?,
            world_name: None,
            include_command_line_api: None,
            run_immediately: None,
        })
        .map_err(to_browser_error)?;

        Ok(())
    }

    fn navigator_consistency_script(&self) -> BrowserAutomationResult<String> {
        let languages = serde_json::to_string(&self.languages)
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
        let platform = serde_json::to_string(&self.platform)
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
        Ok(format!(
            r#"
            (() => {{
              const define = (target, key, value) => {{
                try {{
                  Object.defineProperty(target, key, {{
                    get: () => value,
                    configurable: true
                  }});
                }} catch (_) {{}}
              }};

              // Navigator properties
              define(Navigator.prototype, "languages", {languages});
              define(Navigator.prototype, "platform", {platform});
              define(Navigator.prototype, "hardwareConcurrency", {hardware_concurrency});
              define(Navigator.prototype, "deviceMemory", {device_memory});
              define(Navigator.prototype, "maxTouchPoints", 0);

              // Remove webdriver signal
              define(Navigator.prototype, "webdriver", undefined);
              delete navigator.__proto__.webdriver;

              // Screen metrics
              define(Screen.prototype, "width", {screen_width});
              define(Screen.prototype, "height", {screen_height});
              define(Screen.prototype, "availWidth", {screen_width});
              define(Screen.prototype, "availHeight", {screen_height});
              define(Screen.prototype, "colorDepth", {color_depth});
              define(Screen.prototype, "pixelDepth", {screen_depth});

              // Plugin spoofing — realistic Chrome plugin list
              const pluginData = [
                {{ name: "Chrome PDF Plugin", filename: "internal-pdf-viewer", description: "Portable Document Format" }},
                {{ name: "Chrome PDF Viewer", filename: "mhjfbmdgcfjbbpaeojofohoefgiehjai", description: "" }},
                {{ name: "Native Client", filename: "internal-nacl-plugin", description: "" }}
              ];
              const pluginList = pluginData.map(p => {{
                const plugin = Object.create(Plugin.prototype);
                Object.defineProperties(plugin, {{
                  name: {{ value: p.name, enumerable: true }},
                  filename: {{ value: p.filename, enumerable: true }},
                  description: {{ value: p.description, enumerable: true }},
                  length: {{ value: 0 }}
                }});
                return plugin;
              }});
              Object.defineProperty(pluginList, "length", {{ value: pluginData.length }});
              define(Navigator.prototype, "plugins", pluginList);
              define(Navigator.prototype, "mimeTypes", []);

              // WebGL vendor/renderer
              const getParameterOrig = WebGLRenderingContext.prototype.getParameter;
              WebGLRenderingContext.prototype.getParameter = function(param) {{
                const ext = this.getExtension("WEBGL_debug_renderer_info");
                if (ext) {{
                  if (param === ext.UNMASKED_VENDOR_WEBGL) return "{webgl_vendor}";
                  if (param === ext.UNMASKED_RENDERER_WEBGL) return "{webgl_renderer}";
                }}
                return getParameterOrig.call(this, param);
              }};

              // Canvas noise — subtle per-session fingerprint variation
              const seed = Math.random() * 0.01;
              const toDataURLOrig = HTMLCanvasElement.prototype.toDataURL;
              HTMLCanvasElement.prototype.toDataURL = function(type) {{
                try {{
                  const ctx = this.getContext("2d");
                  if (ctx) {{
                    const imageData = ctx.getImageData(0, 0, this.width, this.height);
                    for (let i = 0; i < imageData.data.length; i += 4) {{
                      imageData.data[i] = Math.max(0, Math.min(255, imageData.data[i] + Math.floor(seed * 10)));
                    }}
                    ctx.putImageData(imageData, 0, 0);
                  }}
                }} catch (_) {{}}
                return toDataURLOrig.call(this, type);
              }};

              // AudioContext fingerprint noise
              const audioCtxOrig = (typeof AudioContext !== "undefined" ? AudioContext : (typeof webkitAudioContext !== "undefined" ? webkitAudioContext : null));
              if (audioCtxOrig) {{
                const createOscillatorOrig = audioCtxOrig.prototype.createOscillator;
                audioCtxOrig.prototype.createOscillator = function() {{
                  const osc = createOscillatorOrig.call(this);
                  const freqOrig = osc.frequency.value;
                  osc.frequency.value = freqOrig + seed * 0.1;
                  return osc;
                }};
              }}
            }})();
            "#,
            hardware_concurrency = self.hardware_concurrency,
            device_memory = self.device_memory,
            screen_width = self.screen_resolution.0,
            screen_height = self.screen_resolution.1,
            color_depth = self.color_depth,
            screen_depth = self.color_depth,
            webgl_vendor = self.webgl_vendor,
            webgl_renderer = self.webgl_renderer,
        ))
    }
}

/// Static browser profile template used by [`SELECTION_POOL`].
#[apply(plain_copy_eq)]
pub struct FingerprintProfileTemplate {
    /// User agent string.
    pub user_agent: &'static str,
    /// Viewport width and height.
    pub viewport: (u32, u32),
    /// Browser language list.
    pub languages: &'static [&'static str],
    /// Navigator platform value.
    pub platform: &'static str,
    /// Informational WebGL vendor value.
    pub webgl_vendor: &'static str,
    /// Informational WebGL renderer value.
    pub webgl_renderer: &'static str,
    /// Screen width and height.
    pub screen_resolution: (u32, u32),
    /// Screen color depth in bits.
    pub color_depth: u32,
    /// IANA timezone identifier.
    pub timezone: &'static str,
    /// Hardware concurrency value.
    pub hardware_concurrency: u8,
    /// Device memory value.
    pub device_memory: u8,
}

impl FingerprintProfileTemplate {
    /// Converts this static template into an owned [`FingerprintProfile`].
    #[must_use]
    pub fn to_profile(self) -> FingerprintProfile {
        FingerprintProfile {
            user_agent: self.user_agent.to_owned(),
            viewport: self.viewport,
            languages: self
                .languages
                .iter()
                .map(|language| (*language).to_owned())
                .collect(),
            platform: self.platform.to_owned(),
            webgl_vendor: self.webgl_vendor.to_owned(),
            webgl_renderer: self.webgl_renderer.to_owned(),
            screen_resolution: self.screen_resolution,
            color_depth: self.color_depth,
            timezone: self.timezone.to_owned(),
            hardware_concurrency: self.hardware_concurrency,
            device_memory: self.device_memory,
        }
    }
}

macro_rules! profile_template {
    (
        $ua:literal,
        $viewport:expr,
        $languages:expr,
        $platform:literal,
        $vendor:literal,
        $renderer:literal,
        $screen:expr,
        $timezone:literal,
        $hardware:literal,
        $memory:literal
    ) => {
        FingerprintProfileTemplate {
            user_agent: $ua,
            viewport: $viewport,
            languages: $languages,
            platform: $platform,
            webgl_vendor: $vendor,
            webgl_renderer: $renderer,
            screen_resolution: $screen,
            color_depth: 24,
            timezone: $timezone,
            hardware_concurrency: $hardware,
            device_memory: $memory,
        }
    };
}

/// Built-in browser profile templates for session randomization.
pub const SELECTION_POOL: &[FingerprintProfileTemplate] = &[
    profile_template!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        (1365, 768),
        &["en-US", "en"],
        "Win32",
        "Google Inc. (Intel)",
        "ANGLE (Intel, Intel(R) UHD Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)",
        (1366, 768),
        "America/New_York",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
        (1440, 900),
        &["en-US", "en"],
        "Win32",
        "Google Inc. (Intel)",
        "ANGLE (Intel, Intel(R) UHD Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)",
        (1440, 900),
        "America/Chicago",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
        (1536, 864),
        &["en-US", "en"],
        "Win32",
        "Google Inc. (Intel)",
        "ANGLE (Intel, Intel(R) UHD Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)",
        (1536, 864),
        "America/Denver",
        12,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
        (1600, 900),
        &["en-US", "en"],
        "Win32",
        "Google Inc. (Intel)",
        "ANGLE (Intel, Intel(R) UHD Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)",
        (1600, 900),
        "America/Los_Angeles",
        8,
        16
    ),
    profile_template!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        (1920, 1080),
        &["en-US", "en"],
        "Win32",
        "Google Inc. (Intel)",
        "ANGLE (Intel, Intel(R) UHD Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)",
        (1920, 1080),
        "America/New_York",
        16,
        16
    ),
    profile_template!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
        (1366, 768),
        &["en-US", "en"],
        "Win32",
        "Google Inc. (NVIDIA)",
        "ANGLE (NVIDIA, NVIDIA GeForce GTX 1650 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        (1366, 768),
        "America/New_York",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36 Edg/121.0.0.0",
        (1440, 900),
        &["en-US", "en"],
        "Win32",
        "Google Inc. (NVIDIA)",
        "ANGLE (NVIDIA, NVIDIA GeForce GTX 1650 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        (1440, 900),
        "America/Chicago",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.0.0",
        (1536, 864),
        &["en-US", "en"],
        "Win32",
        "Google Inc. (NVIDIA)",
        "ANGLE (NVIDIA, NVIDIA GeForce GTX 1650 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        (1536, 864),
        "America/Denver",
        12,
        16
    ),
    profile_template!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0",
        (1366, 768),
        &["en-US", "en"],
        "Win32",
        "Mozilla",
        "Mozilla",
        (1366, 768),
        "America/New_York",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:123.0) Gecko/20100101 Firefox/123.0",
        (1920, 1080),
        &["en-US", "en"],
        "Win32",
        "Mozilla",
        "Mozilla",
        (1920, 1080),
        "America/Los_Angeles",
        16,
        16
    ),
    profile_template!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        (1440, 900),
        &["en-US", "en"],
        "MacIntel",
        "Google Inc. (Apple)",
        "ANGLE (Apple, Apple M1, OpenGL 4.1)",
        (1440, 900),
        "America/Los_Angeles",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
        (1512, 982),
        &["en-US", "en"],
        "MacIntel",
        "Google Inc. (Apple)",
        "ANGLE (Apple, Apple M1, OpenGL 4.1)",
        (1512, 982),
        "America/New_York",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_2) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
        (1728, 1117),
        &["en-US", "en"],
        "MacIntel",
        "Google Inc. (Apple)",
        "ANGLE (Apple, Apple M1, OpenGL 4.1)",
        (1728, 1117),
        "America/Chicago",
        10,
        16
    ),
    profile_template!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
        (1920, 1080),
        &["en-GB", "en"],
        "MacIntel",
        "Google Inc. (Apple)",
        "ANGLE (Apple, Apple M1, OpenGL 4.1)",
        (1920, 1080),
        "Europe/London",
        10,
        16
    ),
    profile_template!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36 Edg/121.0.0.0",
        (1440, 900),
        &["en-US", "en"],
        "MacIntel",
        "Google Inc. (Apple)",
        "ANGLE (Apple, Apple M2, OpenGL 4.1)",
        (1440, 900),
        "America/Los_Angeles",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_2) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.0.0",
        (1512, 982),
        &["en-GB", "en"],
        "MacIntel",
        "Google Inc. (Apple)",
        "ANGLE (Apple, Apple M2, OpenGL 4.1)",
        (1512, 982),
        "Europe/London",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.2; rv:123.0) Gecko/20100101 Firefox/123.0",
        (1440, 900),
        &["en-US", "en"],
        "MacIntel",
        "Mozilla",
        "Mozilla",
        (1440, 900),
        "America/New_York",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.3; rv:124.0) Gecko/20100101 Firefox/124.0",
        (1728, 1117),
        &["en-US", "en"],
        "MacIntel",
        "Mozilla",
        "Mozilla",
        (1728, 1117),
        "America/Los_Angeles",
        10,
        16
    ),
    profile_template!(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        (1366, 768),
        &["en-US", "en"],
        "Linux x86_64",
        "Google Inc. (AMD)",
        "ANGLE (AMD, AMD Radeon Graphics, OpenGL 4.6)",
        (1366, 768),
        "Europe/Berlin",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
        (1440, 900),
        &["en-GB", "en"],
        "Linux x86_64",
        "Google Inc. (AMD)",
        "ANGLE (AMD, AMD Radeon Graphics, OpenGL 4.6)",
        (1440, 900),
        "Europe/London",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
        (1536, 864),
        &["en-US", "en"],
        "Linux x86_64",
        "Google Inc. (AMD)",
        "ANGLE (AMD, AMD Radeon Graphics, OpenGL 4.6)",
        (1536, 864),
        "America/New_York",
        12,
        16
    ),
    profile_template!(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36 Edg/121.0.0.0",
        (1366, 768),
        &["en-US", "en"],
        "Linux x86_64",
        "Google Inc. (Intel)",
        "ANGLE (Intel, Mesa Intel(R) UHD Graphics 620, OpenGL 4.6)",
        (1366, 768),
        "Europe/Berlin",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (X11; Linux x86_64; rv:123.0) Gecko/20100101 Firefox/123.0",
        (1366, 768),
        &["en-GB", "en"],
        "Linux x86_64",
        "Mozilla",
        "Mozilla",
        (1366, 768),
        "Europe/London",
        8,
        8
    ),
    profile_template!(
        "Mozilla/5.0 (X11; Linux x86_64; rv:124.0) Gecko/20100101 Firefox/124.0",
        (1920, 1080),
        &["en-US", "en"],
        "Linux x86_64",
        "Mozilla",
        "Mozilla",
        (1920, 1080),
        "Europe/Berlin",
        16,
        16
    ),
];

fn platform_matches(requested: &str, actual: &str) -> bool {
    let requested = requested.trim().to_ascii_lowercase();
    let actual = actual.trim().to_ascii_lowercase();
    requested == actual
        || matches!(
            (requested.as_str(), actual.as_str()),
            ("windows", "win32")
                | ("win", "win32")
                | ("macos", "macintel")
                | ("mac", "macintel")
                | ("linux", "linux x86_64")
        )
}

fn to_browser_error(error: impl ToString) -> BrowserAutomationError {
    BrowserAutomationError::Browser(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn random_should_generate_realistic_profile_values() {
        let profile = FingerprintProfile::random();

        assert!(
            profile.user_agent.contains("Mozilla/5.0")
                && profile.viewport.0 >= 1280
                && profile.viewport.1 >= 720
                && !profile.languages.is_empty()
                && matches!(
                    profile.platform.as_str(),
                    "Win32" | "MacIntel" | "Linux x86_64"
                )
                && profile.color_depth == 24
                && matches!(profile.hardware_concurrency, 8 | 10 | 12 | 16)
                && matches!(profile.device_memory, 8 | 16),
            "profile should stay inside the built-in realistic desktop ranges: {profile:?}"
        );
    }

    #[test]
    fn for_platform_should_keep_requested_platform_family() {
        let profile = FingerprintProfile::for_platform("macos");

        assert_eq!(profile.platform, "MacIntel");
    }

    #[test]
    fn selection_pool_should_have_distinct_profiles() {
        let unique = SELECTION_POOL
            .iter()
            .map(|profile| {
                (
                    profile.user_agent,
                    profile.viewport,
                    profile.screen_resolution,
                    profile.timezone,
                )
            })
            .collect::<HashSet<_>>();

        assert_eq!(unique.len(), SELECTION_POOL.len());
    }

    #[test]
    fn selection_pool_should_contain_at_least_twenty_profiles() {
        assert!(SELECTION_POOL.len() >= 20);
    }
}
