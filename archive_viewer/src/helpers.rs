use handlebars_iron::handlebars::*;

pub fn add_helpers(handlebars: &mut Handlebars) {
  handlebars.register_helper("range", box range);
  handlebars.register_helper("eq", box eq);
  handlebars.register_helper("hex", box hex);
  handlebars.register_helper("break_lines", box break_lines);
}

fn range(h: &Helper, r: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
  let param = h.params()[0].value().as_u64().unwrap();
  for i in 0..param {
    rc.push_block_context(&(i + 1));
    h.template().map(|t| t.render(r, rc)).unwrap_or(Ok(())).unwrap();
  }
  Ok(())
}

fn eq(h: &Helper, r: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
  let param_1 = h.params()[0].value();
  let param_2 = h.params()[1].value();
  if param_1 == param_2 {
    h.template().map(|t| t.render(r, rc)).unwrap_or(Ok(())).unwrap();
  }
  Ok(())
}

fn hex(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
  let param = h.params()[0].value().as_u64().unwrap();
  let hex = format!("{:x}", param);
  rc.writer().write_all(hex.as_bytes())?;
  Ok(())
}

fn break_lines(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
  let param = h.params()[0].value().as_str().unwrap();
  let broken = html_escape(param).replace("\n", "<br/>");
  rc.writer().write_all(broken.as_bytes())?;
  Ok(())
}
