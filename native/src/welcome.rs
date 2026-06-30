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
