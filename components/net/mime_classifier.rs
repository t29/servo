use std::io::File;

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
  fn windows_icon_sniffer()->ByteMatcher {
    return ByteMatcher{
      pattern:vec![0x00u8,0x00u8,0x01u8,0x00u8],
      mask:vec![0xFFu8,0xFFu8,0xFFu8,0xFFu8],
      MIME_type:"image/x-icon".to_string(),
      leading_ignore:vec![]}
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
  let matcher = ByteMatcher::windows_icon_sniffer();

  let p = Path::new("./tests/content/test.ico");
  let mut file = File::open(&p);
  let read_result = file.read_to_end();
  match read_result {
    Ok(data) => {
      println!("Data Length {:u}",data.len());
      if !matcher.matches(&data) {
        fail!("Didn't read mime type")
      }
    },
    Err(e) => fail!("Couldn't read from file!")
  }

}


struct MIMEClassifier
{
  //TODO Replace with boxed trait
  byte_matchers: Vec<ByteMatcher>,//TODO should change to Box<MIMEChecker+Send>, but I need to figure out lifetimes first
}

impl MIMEClassifier
{
  fn new()->MIMEClassifier {
     //TODO These should be configured from a settings file
     //     and not hardcoded
     let mut vec = Vec::new();
     vec.push(ByteMatcher::windows_icon_sniffer());
     return MIMEClassifier{byte_matchers:vec};
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
fn test_classify_windows_icon() {
  let classifier = MIMEClassifier::new();

  let p = Path::new("./tests/content/test.ico");
  let mut file = File::open(&p);
  let read_result = file.read_to_end();
  match read_result {
    Ok(data) => {
      
      match classifier.classify(&data)
      {
        Some(x)=>{ if (x!="image/x-icon".to_string()) {
          fail!("Windows Icon parsed incorrectly");
          }
        }
        None=>{fail!("No classification found");}
      }
    }
    Err(e) => {fail!("Couldn't read from file!");}
  }
}

