// ponytail: single-purpose logo markup, expand to full welcome page when needed
#[napi]
pub fn cod_logo_html() -> String {
    format!(
        r#"<div style="display:flex;align-items:center;gap:14px;margin-bottom:16px">
  <div style="width:52px;height:52px;background:#00BCA2;border-radius:14px;display:flex;align-items:center;justify-content:center;font-size:24px;font-weight:800;color:#1a1a1a;flex-shrink:0">COD</div>
  <div><h1 style="font-size:26px;font-weight:700;color:#fff;margin:0">Welcome to COD</h1>
  <span style="font-size:13px;color:#969696">Your streamlined code editor — fast, focused, yours.</span></div>
</div>"#
    )
}

// ponytail: static markup, extend with links when navigation support needed
#[napi]
pub fn cod_about_html(version: String, commit: String, date: String) -> String {
    format!(
        r#"<div style="text-align:center;padding:8px 0">
  <div style="width:72px;height:72px;background:#00BCA2;border-radius:18px;display:flex;align-items:center;justify-content:center;font-size:34px;font-weight:800;color:#1a1a1a;margin:0 auto 16px">COD</div>
  <h2 style="font-size:20px;font-weight:700;color:#fff;margin:0 0 2px">COD Editor</h2>
  <div style="font-size:12px;color:#969696;font-family:monospace;margin-bottom:4px">Version {version}</div>
  <div style="font-size:11px;color:#969696;margin-bottom:20px">Build {date} · Commit {commit}</div>
  <div style="height:1px;background:#3c3c3c;margin:16px 0"></div>
  <div style="font-size:12px;color:#969696;margin-bottom:20px">
    © COD Contributors. All rights reserved.<br>
    Built on Visual Studio Code — MIT License.
  </div>
</div>"#
    )
}
