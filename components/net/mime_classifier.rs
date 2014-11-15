use std::io::File;
use std::io::fs;
use std::io::fs::PathExtensions;

trait MIMEChecker {
  fn classify(&self, data:&Vec<u8>)->Option<(String,String)>;
}

struct ByteMatcher {
  pattern: Vec<u8>,
  mask: Vec<u8>,
  leading_ignore: Vec<u8>,
  content_type: (String,String)
}

impl ByteMatcher {
  fn matches(&self,data:&Vec<u8>)->bool {

    if data.len() < self.pattern.len() {
      return false;
    }
    //TODO replace with iterators if I ever figure them out...
    let mut i = 0u;
    let max_i = data.len()-self.pattern.len();

    loop {

      if !self.leading_ignore.iter().any(|x| *x == data[i]) { break;}

      i=i+1;
      if i>max_i {return false;}
    }

    for j in range(0u,self.pattern.len()) {
      if (data[i] & self.mask[j])!=
        (self.pattern[j] & self.mask[j]) {
        return false;
      }
      i=i+1;
    }
    return true;
  }
//TODO These should probably be configured not hard coded
  fn image_x_icon()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x00u8,0x00u8,0x01u8,0x00u8],
      mask:vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"x-icon".to_string()),
      leading_ignore:vec![]}
  }
  fn image_x_icon_cursor()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x00u8,0x00u8,0x02u8,0x00u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"x-icon".to_string()),
      leading_ignore:vec![]
    }
  }
  fn image_bmp()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x42u8,0x4Du8],
      mask:   vec![0xFFu8,0xFFu8],
      content_type:("image".to_string(),"bmp".to_string()),
      leading_ignore:vec![]
    }
  }
  fn image_gif89a()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x47u8,0x49u8,0x46u8,0x38u8,0x39u8,0x61u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"gif".to_string()),
      leading_ignore:vec![]
    }
  }
  fn image_gif87a()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x47u8,0x49u8,0x46u8,0x38u8,0x37u8,0x61u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"gif".to_string()),
      leading_ignore:vec![]
    }
  }
  fn image_webp()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x52u8,0x49u8,0x46u8,0x46u8,0x00u8,0x00u8,0x00u8,0x00u8,
                   0x57u8,0x45u8,0x42u8,0x50u8,0x56u8,0x50u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0x00u8,0x00u8,0x00u8,0x00u8,
                   0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"webp".to_string()),
      leading_ignore:vec![]
    }
  }

  fn image_png()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x89u8,0x50u8,0x4Eu8,0x47u8,0x0Du8,0x0Au8,0x1Au8,0x0Au8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"png".to_string()),
      leading_ignore:vec![]
    }
  }
  fn image_jpeg()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0xFFu8,0xD8u8,0xFFu8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"jpeg".to_string()),
      leading_ignore:vec![]
    }
  }
  fn video_webm()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x1Au8,0x45u8,0xDFu8,0xA3u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("video".to_string(),"webm".to_string()),
      leading_ignore:vec![]
    }
  }
  fn audio_basic()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x2Eu8,0x73u8,0x6Eu8,0x64u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("audio".to_string(),"basic".to_string()),
      leading_ignore:vec![]
    }
  }
  fn audio_aiff()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x46u8,0x4Fu8,0x52u8,0x4Du8,0x00u8,0x00u8,0x00u8,0x00u8,
                   0x41u8,0x49u8,0x46u8,0x46u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0x00u8,0x00u8,0x00u8,0x00u8,
                   0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("audio".to_string(),"aiff".to_string()),
      leading_ignore:vec![]
    }
  }
  fn audio_mpeg()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x49u8,0x44u8,0x33u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8],
      content_type:("audio".to_string(),"mpeg".to_string()),
      leading_ignore:vec![]
    }
  }
  fn application_ogg()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x4Fu8,0x67u8,0x67u8,0x53u8,0x00u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("application".to_string(),"ogg".to_string()),
      leading_ignore:vec![]
    }
  }
  fn audio_midi()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x4Du8,0x54u8,0x68u8,0x64u8,0x00u8,0x00u8,0x00u8,0x06u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("audio".to_string(),"midi".to_string()),
      leading_ignore:vec![]
    }
  }
  fn video_avi()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x52u8,0x49u8,0x46u8,0x46u8,0x00u8,0x00u8,0x00u8,0x00u8,
                   0x41u8,0x56u8,0x49u8,0x20u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0x00u8,0x00u8,0x00u8,0x00u8,
                   0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("video".to_string(),"avi".to_string()),
      leading_ignore:vec![]
    }
  }
  fn audio_wave()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x52u8,0x49u8,0x46u8,0x46u8,0x00u8,0x00u8,0x00u8,0x00u8,
                   0x57u8,0x41u8,0x56u8,0x45u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0x00u8,0x00u8,0x00u8,0x00u8,
                   0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("audio".to_string(),"wave".to_string()),
      leading_ignore:vec![]
    }
  }
  // doctype terminated with Tag terminating (TT) Byte: 0x20 (SP)
  fn text_html_doctype_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x21u8,0x44u8,0x4Fu8,0x43u8,0x54u8,0x59u8,0x50u8,
                   0x45u8,0x20u8,0x48u8,0x54u8,0x4Du8,0x4Cu8,0x20u8],
      mask:   vec![0xFFu8,0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,
                   0xDFu8,0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // doctype terminated with Tag terminating (TT) Byte: 0x3E (">")
  fn text_html_doctype_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x21u8,0x44u8,0x4Fu8,0x43u8,0x54u8,0x59u8,0x50u8,
                   0x45u8,0x20u8,0x48u8,0x54u8,0x4Du8,0x4Cu8,0x3Eu8],
      mask:   vec![0xFFu8,0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,
                   0xDFu8,0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // HTML terminated with Tag terminating (TT) Byte: 0x20 (SP)
  fn text_html_page_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x48u8,0x54u8,0x4Du8,0x4Cu8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // HTML terminated with Tag terminating (TT) Byte: 0x3E (">")
  fn text_html_page_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x48u8,0x54u8,0x4Du8,0x4Cu8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // head terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_head_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x48u8,0x45u8,0x41u8,0x44u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // head terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_head_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x48u8,0x45u8,0x41u8,0x44u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // script terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_script_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x53u8,0x43u8,0x52u8,0x49u8,0x50u8,0x54u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // script terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_script_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x53u8,0x43u8,0x52u8,0x49u8,0x50u8,0x54u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // iframe terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_iframe_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x49u8,0x46u8,0x52u8,0x41u8,0x4Du8,0x45u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // iframe terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_iframe_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x49u8,0x46u8,0x52u8,0x41u8,0x4Du8,0x45u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // h1 terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_h1_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x48u8,0x31u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xFFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // h1 terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_h1_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x48u8,0x31u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xFFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // div terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_div_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x44u8,0x49u8,0x56u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // div terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_div_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x44u8,0x49u8,0x56u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // font terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_font_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x46u8,0x4Fu8,0x4Eu8,0x54u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // font terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_font_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x46u8,0x4Fu8,0x4Eu8,0x54u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // table terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_table_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x54u8,0x41u8,0x42u8,0x4Cu8,0x45u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // table terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_table_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x54u8,0x41u8,0x42u8,0x4Cu8,0x45u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // a terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_a_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x41u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // a terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_a_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x41u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // style terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_style_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x53u8,0x54u8,0x59u8,0x4Cu8,0x45u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // style terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_style_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x53u8,0x54u8,0x59u8,0x4Cu8,0x45u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // title terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_title_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x54u8,0x49u8,0x54u8,0x4Cu8,0x45u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // title terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_title_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x54u8,0x49u8,0x54u8,0x4Cu8,0x45u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // b terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_b_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x42u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // b terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_b_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x42u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // body terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_body_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x42u8,0x4Fu8,0x44u8,0x59u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // body terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_body_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x42u8,0x4Fu8,0x44u8,0x59u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // br terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_br_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x42u8,0x52u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // br terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_br_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x42u8,0x52u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // p terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_p_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x50u8,0x20u8],
      mask:   vec![0xFFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // p terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_p_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x50u8,0x3Eu8],
      mask:   vec![0xFFu8,0xDFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // comment terminated with Tag Terminating (TT) Byte: 0x20 (SP)
  fn text_html_comment_20()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x21u8,0x2Du8,0x2Du8,0x20u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // comment terminated with Tag Terminating (TT) Byte: 0x3E (">")
  fn text_html_comment_3E()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x21u8,0x2Du8,0x2Du8,0x3Eu8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // XML
  fn text_xml()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x3Cu8,0x3Fu8,0x78u8,0x6Du8,0x6Cu8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
  // PDF
  fn text_pdf()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x25u8,0x50u8,0x44u8,0x46u8,0x2Du8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("text".to_string(),"html".to_string()),
      leading_ignore:vec![]
    }
  }
}

impl MIMEChecker for ByteMatcher {
  fn classify(&self, data:&Vec<u8>)->Option<(String,String)>
  {
   return if self.matches(data) {
      Some(self.content_type.clone())
    } else {
      None
    };
  }
}


#[test]
fn test_sniff_windows_icon() {
  let matcher = ByteMatcher::image_x_icon();

  let p = Path::new("./tests/content/parsable_mime/image/x-icon/test.ico");
  let mut file = File::open(&p);
  let read_result = file.read_to_end();
  match read_result {
    Ok(data) => {
      println!("Data Length {:u}",data.len());
      if !matcher.matches(&data) {
        panic!("Didn't read mime type")
      }
    },
    Err(e) => panic!("Couldn't read from file with error {}",e)
  }

}

#[test]
fn test_sniff_windows_cursor() {
  let matcher = ByteMatcher::image_x_icon_cursor();

  let p = Path::new("./tests/content/parsable_mime/image/x-icon/test_cursor.ico");
  let mut file = File::open(&p);
  let read_result = file.read_to_end();
  match read_result {
    Ok(data) => {
      println!("Data Length {:u}",data.len());
      if !matcher.matches(&data) {
        panic!("Didn't read mime type")
      }
    },
    Err(e) => panic!("Couldn't read from file with error {}",e)
  }
}

#[test]
fn test_sniff_windows_bmp() {
  let matcher = ByteMatcher::image_bmp();

  let p = Path::new("./tests/content/parsable_mime/image/bmp/test.bmp");
  let mut file = File::open(&p);
  let read_result = file.read_to_end();
  match read_result {
    Ok(data) => {
      println!("Data Length {:u}",data.len());
      if !matcher.matches(&data) {
        panic!("Didn't read mime type")
      }
    },
    Err(e) => panic!("Couldn't read from file with error {}",e)
  }
}

struct MIMEClassifier
{
  //TODO Replace with boxed trait
  byte_matchers: Vec<Box<MIMEChecker+Send>>,
}

impl MIMEClassifier
{
  fn new()->MIMEClassifier {
     //TODO These should be configured from a settings file
     //     and not hardcoded

     let mut ret = MIMEClassifier{byte_matchers:Vec::new()};
     ret.byte_matchers.push(box ByteMatcher::image_x_icon());
     ret.byte_matchers.push(box ByteMatcher::image_x_icon_cursor());
     ret.byte_matchers.push(box ByteMatcher::image_bmp());
     ret.byte_matchers.push(box ByteMatcher::image_gif89a());
     ret.byte_matchers.push(box ByteMatcher::image_gif87a());
     ret.byte_matchers.push(box ByteMatcher::image_webp());
     ret.byte_matchers.push(box ByteMatcher::image_png());
     ret.byte_matchers.push(box ByteMatcher::image_jpeg());
     ret.byte_matchers.push(box ByteMatcher::video_webm());
     ret.byte_matchers.push(box ByteMatcher::audio_basic());
     ret.byte_matchers.push(box ByteMatcher::audio_aiff());
     ret.byte_matchers.push(box ByteMatcher::audio_mpeg());
     ret.byte_matchers.push(box ByteMatcher::application_ogg());
     ret.byte_matchers.push(box ByteMatcher::audio_midi());
     ret.byte_matchers.push(box ByteMatcher::video_avi());
     ret.byte_matchers.push(box ByteMatcher::audio_wave());
     return ret;

  }

  fn classify(&self,data:&Vec<u8>)->Option<(String,String)> {
    for matcher in self.byte_matchers.iter()
    {
      match matcher.classify(data)
      {
        Some(mime)=>{ return Some(mime);}
        None=>{}
      }
    }
    return None;
  }

}

#[test]
fn test_classify_parsable_content_types() {
  let classifier = MIMEClassifier::new();
  let mimes_path= Path::new("./tests/content/parsable_mime/");

  match fs::walk_dir(&mimes_path) {
    Err(why) => panic!("! {}", why.kind),
    Ok(mut paths) => for p in paths {
      if p.is_file() {
        match p.path_relative_from(&mimes_path) {
          Some(rel_path)=>{
            let dir_str = match rel_path.dirname_str() {
               Some(nm) => nm.to_string(),
               None=>"".to_string()};
            let ss: Vec<&str> = dir_str.as_slice().split('/').collect();

            let subtype = ss[1].to_string();
            let type_ = ss[0].to_string();

            let mut file = File::open(&p);
            let read_result = file.read_to_end();
            match read_result {
              Ok(data) => {
                match classifier.classify(&data)
                {
                  Some(mime)=>{
                    let parsed_type=mime.ref0().clone();
                    let parsed_subtp=mime.ref1().clone();
                     if (parsed_type!=type_)||(parsed_subtp!=subtype) {
                      panic!("File {} parsed incorrectly should be {}/{}, parsed as {}/{}",rel_path.as_str(),type_,subtype,parsed_type,parsed_subtp);
                    }
                  }
                  None=>{panic!("No classification found for {}",rel_path.as_str());}
                }
              }
              Err(e) => {panic!("Couldn't read from file {} with error {}",p.as_str(),e);}
            }
          }
          None=>{panic!("Couldn't conver to relative path");}
        }
      }
    }
  }
}
