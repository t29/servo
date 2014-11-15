use std::io::File;
use std::io::fs;
use std::io::fs::PathExtensions;

trait MIMEChecker {
  fn classify(&self, data:&Vec<u8>)->Option<String>;
}

struct ByteMatcher {
  pattern: Vec<u8>,
  mask: Vec<u8>,
  leading_ignore: Vec<u8>,
  MIME_type: String,
}

impl ByteMatcher {
  fn matches(&self,data:&Vec<u8>)->bool {

    if (data.len() < self.pattern.len()) {
      return false;
    }
    //TODO replace with iterators if I ever figure them out...
    let mut i = 0u;
    let max_i = data.len()-self.pattern.len();   
    
    loop {
      
      if (!self.leading_ignore.iter().any(|x|
        *x == data[i])) { break;}
        
      i=i+1;
      if (i>max_i) {return false;}
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
      MIME_type:"image/x-icon".to_string(),
      leading_ignore:vec![]}
  }
  fn windows_cursor()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x00u8,0x00u8,0x02u8,0x00u8],
      mask:vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      MIME_type:"image/x-icon".to_string(),
      leading_ignore:vec![]
    }
  }
  fn windows_bmp()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x42u8,0x4Du8],
      mask:vec![0xFFu8,0xFFu8],
      MIME_type:"image/bmp".to_string(),
      leading_ignore:vec![]
    }
  }
}

impl MIMEChecker for ByteMatcher {
  fn classify(&self, data:&Vec<u8>)->Option<String>
  {
   return if self.matches(data) {
      Some(self.MIME_type.clone()) 
    } else {
      None
    };
  }
}


#[test]
fn test_sniff_windows_icon() {
  let matcher = ByteMatcher::windows_icon();

  let p = Path::new("./tests/content/parsable_mime/image/x-icon.ico");
  let mut file = File::open(&p);
  let read_result = file.read_to_end();
  match read_result {
    Ok(data) => {
      println!("Data Length {:u}",data.len());
      if !matcher.matches(&data) {
        panic!("Didn't read mime type")
      }
    },
    Err(e) => panic!("Couldn't read from file!")
  }

}

#[test]
fn test_sniff_windows_cursor() {
  let matcher = ByteMatcher::windows_cursor();

  let p = Path::new("./tests/content/parsable_mime/image/x-icon.cursor");
  let mut file = File::open(&p);
  let read_result = file.read_to_end();
  match read_result {
    Ok(data) => {
      println!("Data Length {:u}",data.len());
      if !matcher.matches(&data) {
        panic!("Didn't read mime type")
      }
    },
    Err(e) => panic!("Couldn't read from file!")
  }
}

#[test]
fn test_sniff_windows_bmp() {
  let matcher = ByteMatcher::windows_bmp();

  let p = Path::new("./tests/content/parsable_mime/image/bmp.bmp");
  let mut file = File::open(&p);
  let read_result = file.read_to_end();
  match read_result {
    Ok(data) => {
      println!("Data Length {:u}",data.len());
      if !matcher.matches(&data) {
        panic!("Didn't read mime type")
      }
    },
    Err(e) => panic!("Couldn't read from file!")
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
     
     return ret;
     
  }
  
  fn classify(&self,data:&Vec<u8>)->Option<String> {
    for matcher in self.byte_matchers.iter()
    {
      match matcher.classify(data)
      {
        Some(mime)=>{ return Some(mime.clone());}
        None=>{}
      }
    }
    return None;
  }

}

#[test]
fn test_classify_parsable_mime_types() {
  let classifier = MIMEClassifier::new();
  let mimes_path= Path::new("./tests/content/parsable_mime/");

  match fs::walk_dir(&mimes_path) {
    Err(why) => panic!("! {}", why.kind),
    Ok(mut paths) => for p in paths {
      if p.is_file() {
        match p.path_relative_from(&mimes_path) {
          Some(rel_path)=>{
            let mut path_type = rel_path.clone();
            path_type.set_extension("");
            match path_type.as_str() {
              Some(type_string)=> {
              let mut file = File::open(&p);
              let read_result = file.read_to_end();
              match read_result {
                Ok(data) => {
                  match classifier.classify(&data)
                  {
                    Some(x)=>{ 
                      if (x!=type_string.to_string()) {
                        panic!("Windows Icon parsed incorrectly");
                      }
                    }
                    None=>{panic!("No classification found for {}",rel_path.as_str());}
                  }
                }
                Err(e) => {panic!("Couldn't read from file {}",p.as_str());}
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

