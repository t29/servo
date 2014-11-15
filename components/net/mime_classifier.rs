use std::io::File;
use std::io::fs;
use std::io::fs::PathExtensions;
use std::str;

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
  fn windows_icon()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x00u8,0x00u8,0x01u8,0x00u8],
      mask:vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"x-icon".to_string()),
      leading_ignore:vec![]}
  }
  fn windows_cursor()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x00u8,0x00u8,0x02u8,0x00u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"x-icon".to_string()),
      leading_ignore:vec![]
    }
  }
  fn windows_bmp()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x42u8,0x4Du8],
      mask:   vec![0xFFu8,0xFFu8],
      content_type:("image".to_string(),"bmp".to_string()),
      leading_ignore:vec![]
    }
  }
  fn gif89a()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x47u8,0x49u8,0x46u8,0x38u8,0x39u8,0x61u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"gif".to_string()),
      leading_ignore:vec![]
    }
  }
  fn gif87a()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x47u8,0x49u8,0x46u8,0x38u8,0x37u8,0x61u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"gif".to_string()),
      leading_ignore:vec![]
    }
  }
  fn webp()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x52u8,0x49u8,0x46u8,0x46u8,0x00u8,0x00u8,0x00u8,0x00u8,
                   0x57u8,0x45u8,0x42u8,0x50u8,0x56u8,0x50u8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0x00u8,0x00u8,0x00u8,0x00u8,
                   0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"webp".to_string()),
      leading_ignore:vec![]
    }
  }

  fn png()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x89u8,0x50u8,0x4Eu8,0x47u8,0x0Du8,0x0Au8,0x1Au8,0x0Au8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"png".to_string()),
      leading_ignore:vec![]
    }
  }
  fn jpeg()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0xFFu8,0xD8u8,0xFFu8],
      mask:   vec![0xFFu8,0xFFu8,0xFFu8],
      content_type:("image".to_string(),"jpeg".to_string()),
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
  let matcher = ByteMatcher::windows_icon();

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
  let matcher = ByteMatcher::windows_cursor();

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
  let matcher = ByteMatcher::windows_bmp();

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
     ret.byte_matchers.push(box ByteMatcher::windows_icon());
     ret.byte_matchers.push(box ByteMatcher::windows_cursor());
     ret.byte_matchers.push(box ByteMatcher::windows_bmp());
     ret.byte_matchers.push(box ByteMatcher::gif89a());
     ret.byte_matchers.push(box ByteMatcher::gif87a());
     ret.byte_matchers.push(box ByteMatcher::webp());
     ret.byte_matchers.push(box ByteMatcher::png());
     ret.byte_matchers.push(box ByteMatcher::jpeg());
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

            match rel_path.dirname_str() {
              Some(type_string)=> {
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
              None=>{panic!("Couldn't convert to string");}
            }
          }
          None=>{panic!("Couldn't conver to relative path");}
        }
      }
    }
  }
}

