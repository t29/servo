/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use std::cmp::max;

fn as_string_option(tup:Option<(&'static str, &'static str)>) -> Option<(String,String)> {
    match tup {
        None => {None}
        Some(tp) => {Some((tp.ref0().to_string(),tp.ref1().to_string()))}
    }
}

trait MIMEChecker {
    fn classify(&self, data:&Vec<u8>)->Option<(String, String)>;
}

trait Matches {
    fn matches(&mut self, matches:&[u8])->bool;
}

impl <'a, T: Iterator<&'a u8>+Clone> Matches for T {

    // Matching function that works on an iterator.
    // see if the next matches.len() bytes in data_iterator equal matches
    // move iterator and return true or just return false
    //
    // Params
    // self: an iterator
    // matches: a vector of bytes to match
    //
    // Return
    // true if the next n elements of self match n elements of matches
    // false otherwise
    //
    // Side effects
    // moves the iterator when match is found
    fn matches(&mut self, matches: &[u8]) -> bool {
        for (byte_a, byte_b) in self.clone().take(matches.len()).zip(matches.iter()) {
            if byte_a != byte_b {
                return false;
            }
        }
        self.nth(matches.len());
        true
    }
}

struct ByteMatcher {
    pattern: &'static [u8],
    mask: &'static [u8],
    leading_ignore: &'static [u8],
    content_type: (&'static str,&'static str)
}

impl ByteMatcher {
    fn matches(&self, data: &Vec<u8>) -> bool {

        if data.len() < self.pattern.len() {
            return false;
        }
        //TODO replace with iterators if I ever figure them out...
        let mut i = 0u;
        let max_i = data.len()-self.pattern.len();

        loop {

            if !self.leading_ignore.iter().any(|x| *x == data[i]) { break;}

            i=i + 1;
            if i > max_i {return false;}
        }

        for j in range(0u,self.pattern.len()) {            
            if (data[i + j] & self.mask[j]) != (self.pattern[j] & self.mask[j]) { 
                return false; 
            }
        }
        return true;
    }
}

impl MIMEChecker for ByteMatcher {
    fn classify(&self, data:&Vec<u8>) -> Option<(String, String)> {
        return if self.matches(data) {
            Some((self.content_type.val0().to_string(),
              self.content_type.val1().to_string()))
        } else {
            None
        };
    }
}

struct Mp4Matcher;

impl Mp4Matcher {
    fn matches(&self,data:&Vec<u8>) -> bool {
        if data.len() < 12 {return false;}
        let box_size = ((data[0] as u32) << 3 | (data[1] as u32) << 2 |
          (data[2] as u32) << 1 | (data[3] as u32)) as uint;
        if (data.len() < box_size) || (box_size % 4 != 0) {return false;}
        //TODO replace with iterators
        let ftyp = [0x66, 0x74, 0x79, 0x70];
        let mp4 =  [0x6D, 0x70, 0x34];

        for i in range(4u,8u) {
            if data[i] != ftyp[i - 4] {
                return false;
            }
        }
        let mut all_match = true;
        for i in range(8u,11u) {
            if data[i]!=mp4[i - 8u] {all_match = false; break;}
        }
        if all_match {return true;}
        let mut bytes_read = 16u;

        while bytes_read < box_size {
            all_match = true;
            for i in range(0u,3u) {
                if mp4[i] != data[i + bytes_read] {all_match = false; break;}
            }
            if all_match {return true;}
            bytes_read=bytes_read + 4;
        }
        return false;
    }

}
impl MIMEChecker for Mp4Matcher {
    fn classify(&self, data:&Vec<u8>) -> Option<(String,String)> {
     return if self.matches(data) {
            Some(("video".to_string(), "mp4".to_string()))
        } else {
            None
        };
    }
}

struct BinaryOrPlaintextClassifier;

impl BinaryOrPlaintextClassifier {
    fn classify_impl(&self, data: &Vec<u8>) -> Option<(&'static str,&'static str)> {
        if (data.len() >=2 &&
            (data[0] == 0xFFu8 && data[1] == 0xFEu8) ||
            (data[0] == 0xFEu8 && data[1] == 0xFFu8)) ||
           (data.len() >= 3 && data[0] == 0xEFu8 && data[1] == 0xBBu8 && data[2] == 0xBFu8)
        {
            return Some(("text","plain"));
        }
        return if data.iter().any(|x| *x<=0x08u8 ||
                                 *x==0x0Bu8 ||
                                 (*x>=0x0Eu8 && *x <= 0x1Au8) ||
                                 (*x>=0x1Cu8 && *x <= 0x1Fu8)) {
            Some(("application","octet-stream"))
        }
        else {
            Some(("text","plain"))
        }
    }
}
impl MIMEChecker for BinaryOrPlaintextClassifier {
    fn classify(&self, data: &Vec<u8>) -> Option<(String, String)> {
        return ::as_string_option(self.classify_impl(data));
    }
}
struct GroupedClassifier {
   byte_matchers: Vec<Box<MIMEChecker + Send>>,
}
impl GroupedClassifier {
    fn push(&mut self,checker:Box<MIMEChecker+Send>)
    {
        self.byte_matchers.push(checker);
    }
    fn new() -> GroupedClassifier {
        return GroupedClassifier{byte_matchers:Vec::new()};
    }
    fn image_classifer() -> GroupedClassifier {
        let mut ret = GroupedClassifier::new();
        ret.push(box ByteMatcher::image_x_icon());
        ret.push(box ByteMatcher::image_x_icon_cursor());
        ret.push(box ByteMatcher::image_bmp());
        ret.push(box ByteMatcher::image_gif89a());
        ret.push(box ByteMatcher::image_gif87a());
        ret.push(box ByteMatcher::image_webp());
        ret.push(box ByteMatcher::image_png());
        ret.push(box ByteMatcher::image_jpeg());

        return ret;
    }
    fn audio_video_classifer() -> GroupedClassifier {
        let mut ret = GroupedClassifier::new();
        ret.push(box ByteMatcher::video_webm());
        ret.push(box ByteMatcher::audio_basic());
        ret.push(box ByteMatcher::audio_aiff());
        ret.push(box ByteMatcher::audio_mpeg());
        ret.push(box ByteMatcher::application_ogg());
        ret.push(box ByteMatcher::audio_midi());
        ret.push(box ByteMatcher::video_avi());
        ret.push(box ByteMatcher::audio_wave());
        ret.byte_matchers.push(box Mp4Matcher);
        return ret;
    }
    fn scriptable_classifier() -> GroupedClassifier {
        let mut ret = GroupedClassifier::new();
        ret.push(box ByteMatcher::text_html_doctype_20());
        ret.push(box ByteMatcher::text_html_doctype_3e());
        ret.push(box ByteMatcher::text_html_page_20());
        ret.push(box ByteMatcher::text_html_page_3e());
        ret.push(box ByteMatcher::text_html_head_20());
        ret.push(box ByteMatcher::text_html_head_3e());
        ret.push(box ByteMatcher::text_html_script_20());
        ret.push(box ByteMatcher::text_html_script_3e());
        ret.push(box ByteMatcher::text_html_iframe_20());
        ret.push(box ByteMatcher::text_html_iframe_3e());
        ret.push(box ByteMatcher::text_html_h1_20());
        ret.push(box ByteMatcher::text_html_h1_3e());
        ret.push(box ByteMatcher::text_html_div_20());
        ret.push(box ByteMatcher::text_html_div_3e());
        ret.push(box ByteMatcher::text_html_font_20());
        ret.push(box ByteMatcher::text_html_font_3e());
        ret.push(box ByteMatcher::text_html_table_20());
        ret.push(box ByteMatcher::text_html_table_3e());
        ret.push(box ByteMatcher::text_html_a_20());
        ret.push(box ByteMatcher::text_html_a_3e());
        ret.push(box ByteMatcher::text_html_style_20());
        ret.push(box ByteMatcher::text_html_style_3e());
        ret.push(box ByteMatcher::text_html_title_20());
        ret.push(box ByteMatcher::text_html_title_3e());
        ret.push(box ByteMatcher::text_html_b_20());
        ret.push(box ByteMatcher::text_html_b_3e());
        ret.push(box ByteMatcher::text_html_body_20());
        ret.push(box ByteMatcher::text_html_body_3e());
        ret.push(box ByteMatcher::text_html_br_20());
        ret.push(box ByteMatcher::text_html_br_3e());
        ret.push(box ByteMatcher::text_html_p_20());
        ret.push(box ByteMatcher::text_html_p_3e());
        ret.push(box ByteMatcher::text_html_comment_20());
        ret.push(box ByteMatcher::text_html_comment_3e());
        ret.push(box ByteMatcher::text_xml());
        ret.push(box ByteMatcher::application_pdf());
        return ret;
    }
    fn plaintext_classifier() -> GroupedClassifier {
        let mut ret = GroupedClassifier::new();
        ret.push(box ByteMatcher::text_plain_utf_8_bom());
        ret.push(box ByteMatcher::text_plain_utf_16le_bom());
        ret.push(box ByteMatcher::text_plain_utf_16be_bom());
        ret.push(box ByteMatcher::application_postscript());
        return ret;
    }
    fn archive_classifier() -> GroupedClassifier {
        let mut ret = GroupedClassifier::new();
        ret.push(box ByteMatcher::application_x_gzip());
        ret.push(box ByteMatcher::application_zip());
        ret.push(box ByteMatcher::application_x_rar_compressed());
        return ret;
    }
    
    fn font_classifier() -> GroupedClassifier {
        let mut ret = GroupedClassifier::new();
        ret.push(box ByteMatcher::application_font_woff());
        ret.push(box ByteMatcher::true_type_collection());
        ret.push(box ByteMatcher::open_type());
        ret.push(box ByteMatcher::true_type());
        ret.push(box ByteMatcher::application_vnd_ms_font_object());
        return ret;
    }
}


impl MIMEChecker for GroupedClassifier {
   fn classify(&self,data:&Vec<u8>) -> Option<(String, String)> {
        for matcher in self.byte_matchers.iter()
        {
            let sniffed_type = matcher.classify(data);
            if sniffed_type.is_some() {return sniffed_type;}
        }
        return None;
   }
}

struct FeedsClassifier;
impl FeedsClassifier {
    fn classify_impl(&self,data:&Vec<u8>) -> Option<(&'static str,&'static str)> {
        let length = data.len();
        let mut data_iterator = data.iter();

        // acceptable byte sequences
        let utf8_bom = [0xEFu8, 0xBBu8, 0xBFu8];

        // can not be feed unless length is > 3
        if length < 3 {
            return None;
        }

        // eat the first three bytes if they are equal to UTF-8 BOM
        data_iterator.matches(utf8_bom);

        // continuously search for next "<" until end of data_iterator
        // TODO: need max_bytes to prevent inadvertently examining html document
        //       eg. an html page with a feed example
        while !data_iterator.find(|&data_iterator| *data_iterator == b'<').is_none() {

            if data_iterator.matches(b"?") {
                // eat until ?>
                while !data_iterator.matches(b"?>") {
                    if data_iterator.next().is_none() {
                        return None;
                    }
                }
            } else if data_iterator.matches(b"!--") {
                // eat until -->
                while !data_iterator.matches(b"-->") {
                    if data_iterator.next().is_none() {
                        return None;
                    }
                }
            } else if data_iterator.matches(b"!") {
                data_iterator.find(|&data_iterator| *data_iterator == b'>');
            } else if data_iterator.matches(b"rss") {
                return Some(("application", "rss+xml"))
            } else if data_iterator.matches(b"feed") {
                return Some(("application", "atom+xml"))
            } else if data_iterator.matches(b"rdf:RDF") {
                while !data_iterator.next().is_none() {
                    if data_iterator.matches(b"http://purl.org/rss/1.0/") {
                        while !data_iterator.next().is_none() {
                            if data_iterator.matches(b"http://www.w3.org/1999/02/22-rdf-syntax-ns#") {
                                return Some(("application", "rss+xml"))
                            }
                        }
                    } else if data_iterator.matches(b"http://www.w3.org/1999/02/22-rdf-syntax-ns#") {
                        while !data_iterator.next().is_none() {
                            if data_iterator.matches(b"http://purl.org/rss/1.0/") {
                                return Some(("application", "rss+xml"))
                            }
                        }
                    }
                }
            }
        }

        return None;
    }
}

impl MIMEChecker for FeedsClassifier {
    fn classify(&self,data:&Vec<u8>) -> Option<(String, String)> {
       return ::as_string_option(self.classify_impl(data));
    }
}

struct MIMEClassifier {
   image_classifier: GroupedClassifier,
   audio_video_classifer: GroupedClassifier,
   scriptable_classifier: GroupedClassifier,
   plaintext_classifier: GroupedClassifier,
   archive_classifer: GroupedClassifier,
   binary_or_plaintext: BinaryOrPlaintextClassifier,
   feeds_classifier: FeedsClassifier
}

impl MIMEClassifier {
    fn new()->MIMEClassifier {
         //TODO These should be configured from a settings file
         //         and not hardcoded
         let ret = MIMEClassifier{
             image_classifier: GroupedClassifier::image_classifer(),
             audio_video_classifer: GroupedClassifier::audio_video_classifer(),
             scriptable_classifier: GroupedClassifier::scriptable_classifier(),
             plaintext_classifier: GroupedClassifier::plaintext_classifier(),
             archive_classifer: GroupedClassifier::archive_classifier(),
             binary_or_plaintext: BinaryOrPlaintextClassifier,
             feeds_classifier: FeedsClassifier
         };
        return ret;

    }
    //some sort of iterator over the classifiers might be better?
    fn sniff_unknown_type(&self, sniff_scriptable:bool, data:&Vec<u8>) ->
      Option<(String,String)> {
        if sniff_scriptable {
            let tp = self.scriptable_classifier.classify(data);
            if tp.is_some() {return tp;}
        }

        let tp = self.plaintext_classifier.classify(data);
        if tp.is_some() {return tp;}

        let tp = self.image_classifier.classify(data);
        if tp.is_some() {return tp;}

        let tp = self.audio_video_classifer.classify(data);
        if tp.is_some() {return tp;}

        let tp = self.archive_classifer.classify(data);
        if tp.is_some() {return tp;}

        self.binary_or_plaintext.classify(data)
    }

    fn sniff_text_or_data(&self,data:&Vec<u8>) -> Option<(String, String)> {
        self.binary_or_plaintext.classify(data)
    }
    fn is_xml(tp:&str,sub_tp:&str) -> bool {
      return match (tp,sub_tp,sub_tp.slice_from(max(sub_tp.len() - "+xml".len(), 0))) {
          (_,_,"+xml") | ("application","xml",_) | ("text","xml",_) => {true}
          _ => {false}
      };
    }
    fn is_html(tp:&str,sub_tp:&str) -> bool { return tp=="text" && sub_tp=="html"; }

    //Performs MIME Type Sniffing Algorithm (section 7)
    fn classify(&self,
                    no_sniff: bool,
                    check_for_apache_bug: bool,
                    supplied_type: &Option<(String,String)>,
                    data:&Vec<u8>) -> Option<(String,String)> {

        match *supplied_type{
            None => {
              return self.sniff_unknown_type(!no_sniff,data);
            }
            Some(ref tup) => {
                let media_type = tup.ref0().as_slice();
                let media_subtype = tup.ref1().as_slice();
                match  (media_type,media_subtype) {
                    ("uknown","unknown") | ("application","uknown") | ("*","*") => {
                        return self.sniff_unknown_type(!no_sniff,data);
                    }
                    _ => {
                        if no_sniff {return supplied_type.clone();}
                        if check_for_apache_bug {
                          return self.sniff_text_or_data(data);
                        }

                        if MIMEClassifier::is_xml(media_type,media_subtype) { 
                          return supplied_type.clone(); 
                        }
                        //Inplied in section 7.3, but flow is not clear
                        if MIMEClassifier::is_html(media_type, media_subtype) { 
                            return self.feeds_classifier.classify(data).
                              or(supplied_type.clone()); 
                         }

                         if media_type == "image" {
                           let tp = self.image_classifier.classify(data);
                           if tp.is_some() { return tp;}
                         }

                         match (media_type,media_subtype) {
                             ("audio",_) | ("video",_) | ("application","ogg") => {
                                 let tp = self.audio_video_classifer.classify(data);
                                 if tp.is_some() { tp;}
                             }
                             _=> {}
                         }
                    }
                }
            }
        }
        return supplied_type.clone();
    }
}

//Contains hard coded byte matchers
//TODO: These should be configured and not hard coded
impl ByteMatcher {
    //A Windows Icon signature
    fn image_x_icon()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x00u8, 0x00u8, 0x01u8, 0x00u8]; P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("image","x-icon"),
            leading_ignore: []}
    }
    //A Windows Cursor signature.
    fn image_x_icon_cursor()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x00u8, 0x00u8, 0x02u8, 0x00u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8];P},
            content_type: ("image","x-icon"),
            leading_ignore: []
        }
    }
    //The string "BM", a BMP signature.
    fn image_bmp()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x42u8, 0x4Du8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8];P},
            content_type: ("image","bmp"),
            leading_ignore: []
        }
    }
    //The string "GIF87a", a GIF signature.
    fn image_gif89a()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x47u8, 0x49u8, 0x46u8, 0x38u8, 0x39u8, 0x61u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8];P},
            content_type: ("image","gif"),
            leading_ignore: []
        }
    }
    //The string "GIF89a", a GIF signature.
    fn image_gif87a()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x47u8, 0x49u8, 0x46u8, 0x38u8, 0x37u8, 0x61u8]; P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("image","gif"),
            leading_ignore: []
        }
    }
    //The string "RIFF" followed by four bytes followed by the string "WEBPVP".
    fn image_webp()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x52u8, 0x49u8, 0x46u8, 0x46u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
              0x57u8, 0x45u8, 0x42u8, 0x50u8, 0x56u8, 0x50u8]; P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
              0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("image","webp"),
            leading_ignore: []
        }
    }
    //An error-checking byte followed by the string "PNG" followed by CR LF SUB LF, the PNG
    //signature.
    fn image_png()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x89u8, 0x50u8, 0x4Eu8, 0x47u8, 0x0Du8, 0x0Au8, 0x1Au8, 0x0Au8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8];P},
            content_type: ("image","png"),
            leading_ignore: []
        }
    }
    // 	The JPEG Start of Image marker followed by the indicator byte of another marker.
    fn image_jpeg()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0xFFu8, 0xD8u8, 0xFFu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8];P},
            content_type: ("image","jpeg"),
            leading_ignore: []
        }
    }
    //The WebM signature. [TODO: Use more bytes?]
    fn video_webm()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x1Au8, 0x45u8, 0xDFu8, 0xA3u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("video","webm"),
            leading_ignore: []
        }
    }
    //The string ".snd", the basic audio signature.
    fn audio_basic()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x2Eu8, 0x73u8, 0x6Eu8, 0x64u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("audio","basic"),
            leading_ignore: []
        }
    }
    //The string "FORM" followed by four bytes followed by the string "AIFF", the AIFF signature.
    fn audio_aiff()->ByteMatcher {
        return ByteMatcher{
            pattern:  {static P:&'static[u8] = &[0x46u8, 0x4Fu8, 0x52u8, 0x4Du8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x41u8, 0x49u8, 0x46u8, 0x46u8];P},
            mask:  {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8];P},
            content_type: ("audio","aiff"),
            leading_ignore: []
        }
    }
    //The string "ID3", the ID3v2-tagged MP3 signature.
    fn audio_mpeg()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x49u8, 0x44u8, 0x33u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("audio","mpeg"),
            leading_ignore: []
        }
    }
    //The string "OggS" followed by NUL, the Ogg container signature.
    fn application_ogg()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x4Fu8, 0x67u8, 0x67u8, 0x53u8, 0x00u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("application","ogg"),
            leading_ignore: []
        }
    }
    //The string "MThd" followed by four bytes representing the number 6 in 32 bits (big-endian),
    //the MIDI signature.
    fn audio_midi()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x4Du8, 0x54u8, 0x68u8, 0x64u8, 0x00u8, 0x00u8, 0x00u8, 0x06u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("audio","midi"),
            leading_ignore: []
        }
    }
    //The string "RIFF" followed by four bytes followed by the string "AVI ", the AVI signature.
    fn video_avi()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x52u8, 0x49u8, 0x46u8, 0x46u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0x41u8, 0x56u8, 0x49u8, 0x20u8]; P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("video","avi"),
            leading_ignore: []
        }
    }
    // 	The string "RIFF" followed by four bytes followed by the string "WAVE", the WAVE signature.
    fn audio_wave()->ByteMatcher {
        return ByteMatcher{
            pattern:  {static P:&'static[u8] = &[0x52u8, 0x49u8, 0x46u8, 0x46u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0x57u8, 0x41u8, 0x56u8, 0x45u8];P},
            mask:  {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8];P},
            content_type: ("audio","wave"),
            leading_ignore: []
        }
    }
    // doctype terminated with Tag terminating (TT) Byte: 0x20 (SP)
    fn text_html_doctype_20()->ByteMatcher {
        return ByteMatcher{
            pattern:  {static P:&'static[u8] = &[0x3Cu8, 0x21u8, 0x44u8, 0x4Fu8, 0x43u8, 0x54u8, 0x59u8, 0x50u8,
                0x45u8, 0x20u8, 0x48u8, 0x54u8, 0x4Du8, 0x4Cu8, 0x20u8]; P},
            mask:  {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8,
                0xDFu8, 0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // doctype terminated with Tag terminating (TT) Byte: 0x3E (">")
    fn text_html_doctype_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x21u8, 0x44u8, 0x4Fu8, 0x43u8, 0x54u8, 0x59u8, 0x50u8,
                0x45u8, 0x20u8, 0x48u8, 0x54u8, 0x4Du8, 0x4Cu8, 0x3Eu8]; P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8,
                0xDFu8, 0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // HTML terminated with Tag terminating (TT) Byte: 0x20 (SP)
    fn text_html_page_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x48u8, 0x54u8, 0x4Du8, 0x4Cu8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // HTML terminated with Tag terminating (TT) Byte: 0x3E (">")
    fn text_html_page_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x48u8, 0x54u8, 0x4Du8, 0x4Cu8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // head terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_head_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x48u8, 0x45u8, 0x41u8, 0x44u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // head terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_head_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x48u8, 0x45u8, 0x41u8, 0x44u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // script terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_script_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x53u8, 0x43u8, 0x52u8, 0x49u8, 0x50u8, 0x54u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // script terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_script_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x53u8, 0x43u8, 0x52u8, 0x49u8, 0x50u8, 0x54u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // iframe terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_iframe_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x49u8, 0x46u8, 0x52u8, 0x41u8, 0x4Du8, 0x45u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // iframe terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_iframe_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x49u8, 0x46u8, 0x52u8, 0x41u8, 0x4Du8, 0x45u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // h1 terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_h1_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x48u8, 0x31u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // h1 terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_h1_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x48u8, 0x31u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // div terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_div_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x44u8, 0x49u8, 0x56u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // div terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_div_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x44u8, 0x49u8, 0x56u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // font terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_font_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x46u8, 0x4Fu8, 0x4Eu8, 0x54u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // font terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_font_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x46u8, 0x4Fu8, 0x4Eu8, 0x54u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // table terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_table_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x54u8, 0x41u8, 0x42u8, 0x4Cu8, 0x45u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // table terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_table_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x54u8, 0x41u8, 0x42u8, 0x4Cu8, 0x45u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // a terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_a_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x41u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // a terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_a_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x41u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // style terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_style_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x53u8, 0x54u8, 0x59u8, 0x4Cu8, 0x45u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // style terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_style_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x53u8, 0x54u8, 0x59u8, 0x4Cu8, 0x45u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // title terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_title_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x54u8, 0x49u8, 0x54u8, 0x4Cu8, 0x45u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // title terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_title_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x54u8, 0x49u8, 0x54u8, 0x4Cu8, 0x45u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // b terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_b_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x42u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // b terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_b_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x42u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // body terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_body_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x42u8, 0x4Fu8, 0x44u8, 0x59u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // body terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_body_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x42u8, 0x4Fu8, 0x44u8, 0x59u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // br terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_br_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x42u8, 0x52u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // br terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_br_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x42u8, 0x52u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // p terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_p_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x50u8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // p terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_p_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x50u8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xDFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // comment terminated with Tag Terminating (TT) Byte: 0x20 (SP)
    fn text_html_comment_20()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x21u8, 0x2Du8, 0x2Du8, 0x20u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    // comment terminated with Tag Terminating (TT) Byte: 0x3E (">")
    fn text_html_comment_3e()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x21u8, 0x2Du8, 0x2Du8, 0x3Eu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("text","html"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
        }
    }
    //The string "<?xml".
    fn text_xml()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x3Cu8, 0x3Fu8, 0x78u8, 0x6Du8, 0x6Cu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("text","xml"),
            leading_ignore:  {static P:&'static[u8] = &[0x09u8, 0x0Au8, 0x0Cu8, 0x0Du8, 0x20u8]; P}
     }
    }
    //The string "%PDF-", the PDF signature.
    fn application_pdf()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x25u8, 0x50u8, 0x44u8, 0x46u8, 0x2Du8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("application","pdf"),
            leading_ignore: []
        }
    }
    //34 bytes followed by the string "LP", the Embedded OpenType signature.
    fn application_vnd_ms_font_object()->ByteMatcher {
        return ByteMatcher{
            pattern:  {static P:&'static[u8] = &[0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0x00u8, 0x00u8, 0x4Cu8, 0x50u8];P},
            mask: { static P:&'static[u8] = &[0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
                0x00u8, 0x00u8, 0xFFu8, 0xFFu8];P},
            content_type: ("application","vnd.ms-fontobject"),
            leading_ignore: []
        }
    }
    //4 bytes representing the version number 1.0, a TrueType signature.
    fn true_type()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x00u8, 0x01u8, 0x00u8, 0x00u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("(TrueType)",""),
            leading_ignore: []
        }
    }
    //The string "OTTO", the OpenType signature.
    fn open_type()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x4Fu8, 0x54u8, 0x54u8, 0x4Fu8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("(OpenType)",""),
            leading_ignore: []
        }
    }
    // 	The string "ttcf", the TrueType Collection signature.
    fn true_type_collection()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x74u8, 0x74u8, 0x63u8, 0x66u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("(TrueType Collection)",""),
            leading_ignore: []
        }
    }
    // 	The string "wOFF", the Web Open Font Format signature.
    fn application_font_woff()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x77u8, 0x4Fu8, 0x46u8, 0x46u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("application","font-woff"),
            leading_ignore: []
        }
    }
    //The GZIP archive signature.
    fn application_x_gzip()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x1Fu8, 0x8Bu8, 0x08u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("application","x-gzip"),
            leading_ignore: []
        }
    }
    //The string "PK" followed by ETX EOT, the ZIP archive signature.
    fn application_zip()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x50u8, 0x4Bu8, 0x03u8, 0x04u8];P},
         mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("application","zip"),
            leading_ignore: []
        }
    }
    //The string "Rar " followed by SUB BEL NUL, the RAR archive signature.
    fn application_x_rar_compressed()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0x52u8, 0x61u8, 0x72u8, 0x20u8, 0x1Au8, 0x07u8, 0x00u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("application","x-rar-compressed"),
            leading_ignore: []
        }
    }
    // 	The string "%!PS-Adobe-", the PostScript signature.
    fn application_postscript()->ByteMatcher {
        return ByteMatcher{
            pattern:  {static P:&'static[u8] = &[0x25u8, 0x21u8, 0x50u8, 0x53u8, 0x2Du8, 0x41u8, 0x64u8, 0x6Fu8,
                0x62u8, 0x65u8, 0x2Du8]; P},
            mask:  {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8,
                0xFFu8, 0xFFu8, 0xFFu8]; P},
            content_type: ("application","postscript"),
            leading_ignore: []
        }
    }
    // 	UTF-16BE BOM
    fn text_plain_utf_16be_bom()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0xFEu8, 0xFFu8, 0x00u8, 0x00u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0x00u8, 0x00u8]; P},
            content_type: ("text","plain"),
            leading_ignore: []
        }
    }
    //UTF-16LE BOM
    fn text_plain_utf_16le_bom()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0xFFu8, 0xFEu8, 0x00u8, 0x00u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0x00u8, 0x00u8]; P},
            content_type: ("text","plain"),
            leading_ignore: []
        }
    }
    //UTF-8 BOM
    fn text_plain_utf_8_bom()->ByteMatcher {
        return ByteMatcher{
            pattern: {static P:&'static[u8] = &[0xEFu8, 0xBBu8, 0xBFu8, 0x00u8];P},
            mask: {static P:&'static[u8] = &[0xFFu8, 0xFFu8, 0xFFu8, 0x00u8]; P},
            content_type: ("text","plain"),
            leading_ignore: []
        }
    }
}

#[cfg(test)]
mod tests {

    use std::io::File;
    use super::Mp4Matcher;
    use super::MIMEClassifier;

    #[test]
    fn test_sniff_mp4() {
        let matcher = Mp4Matcher;

        let p = Path::new("./tests/content/parsable_mime/video/mp4/test.mp4");
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

    #[cfg(test)]
    fn test_classification_full(filename_orig:&Path,type_string:&str,subtype_string:&str,
                                supplied_type:Option<(&'static str,&'static str)>){

        let mut filename = Path::new("./tests/content/parsable_mime/");

        filename.push(filename_orig);

        let classifier = MIMEClassifier::new();

        let mut file = File::open(&filename);
        let read_result = file.read_to_end();
        match read_result {
            Ok(data) => {
                match classifier.classify(false,false,&::as_string_option(supplied_type),&data)
                {
                    Some(mime)=>{
                        let parsed_type=mime.ref0().as_slice();
                        let parsed_subtp=mime.ref1().as_slice();
                         if (parsed_type!=type_string)||
                                (parsed_subtp!=subtype_string) {
                            panic!("File {} parsed incorrectly should be {}/{}, parsed as {}/{}",
                                filename.as_str(),type_string,subtype_string,parsed_type,
                                parsed_subtp);
                        }
                    }
                    None=>{panic!("No classification found for {}",filename.as_str());}
                }
            }
            Err(e) => {panic!("Couldn't read from file {} with error {}",filename.as_str(),e);}
        }
    }

    #[cfg(test)]
    fn test_classification(file:&str,type_string:&str,subtype_string:&str,
                           supplied_type:Option<(&'static str,&'static str)>){
        let mut x = Path::new("./");
        x.push(type_string);
        x.push(subtype_string);
        x.push(file);
        test_classification_full(&x,type_string,subtype_string,supplied_type);
    }

    #[test]
    fn test_classification_x_icon() { test_classification("test.ico","image","x-icon",None); }

    #[test]
    fn test_classification_x_icon_cursor() {
     test_classification("test_cursor.ico","image","x-icon",None);
    }

    #[test]
    fn test_classification_bmp() { test_classification("test.bmp","image","bmp",None); }

    #[test]
    fn test_classification_gif87a() {
        test_classification("test87a.gif","image","gif",None);
    }

    #[test]
    fn test_classification_gif89a() {
        test_classification("test89a.gif","image","gif",None);
    }

    #[test]
    fn test_classification_webp() {
        test_classification("test.webp","image","webp",None);
    }

    #[test]
    fn test_classification_png() {
        test_classification("test.png","image","png",None);
    }

    #[test]
    fn test_classification_jpg() {
        test_classification("test.jpg","image","jpeg",None);
    }

    #[test]
    fn test_classification_webm() {
        test_classification("test.webm","video","webm",None);
    }

    #[test]
    fn test_classification_mp4() {
        test_classification("test.mp4","video","mp4",None);
    }

    #[test]
    fn test_classification_avi() {
        test_classification("test.avi","video","avi",None);
    }

    #[test]
    fn test_classification_basic() {
        test_classification("test.au","audio","basic",None);
    }

    #[test]
    fn test_classification_aiff() {
        test_classification("test.aif","audio","aiff",None);
    }

    #[test]
    fn test_classification_mpeg() {
        test_classification("test.mp3","audio","mpeg",None);
    }

    #[test]
    fn test_classification_midi() {
        test_classification("test.mid","audio","midi",None);
    }

    #[test]
    fn test_classification_wave() {
        test_classification("test.wav","audio","wave",None);
    }

    #[test]
    fn test_classification_ogg() {
        test_classification("small.ogg","application","ogg",None);
    }

    #[test]
    fn test_classification_vsn_ms_fontobject() {
        test_classification("vnd.ms-fontobject","application","vnd.ms-fontobject",None);
    }

    #[test]
    fn test_true_type() {
        test_classification_full(&Path::new("unknown/true_type.ttf"),"(TrueType)","",None);
    }

    #[test]
    fn test_open_type() {
        test_classification_full(&Path::new("unknown/open_type"),"(OpenType)","",None);
    }

    #[test]
    fn test_classification_true_type_collection() {
        test_classification_full(&Path::new("unknown/true_type_collection.ttc"),"(TrueType Collection)","",None);
    }

    #[test]
    fn test_classification_woff() {
        test_classification("test.wof","application","font-woff",None);
    }

    #[test]
    fn test_classification_gzip() {
        test_classification("test.gz","application","x-gzip",None);
    }

    #[test]
    fn test_classification_zip() {
        test_classification("test.zip","application","zip",None);
    }

    #[test]
    fn test_classification_rar() {
        test_classification("test.rar","application","x-rar-compressed",None);
    }

    #[test]
    fn test_text_html_doctype_20() {
        test_classification("text_html_doctype_20.html","text","html",None);
        test_classification("text_html_doctype_20_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_doctype_3e() {
        test_classification("text_html_doctype_3e.html","text","html",None);
        test_classification("text_html_doctype_3e_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_page_20() {
        test_classification("text_html_page_20.html","text","html",None);
        test_classification("text_html_page_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_page_3e() {
        test_classification("text_html_page_3e.html","text","html",None);
        test_classification("text_html_page_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_head_20() {
        test_classification("text_html_head_20.html","text","html",None);
        test_classification("text_html_head_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_head_3e() {
        test_classification("text_html_head_3e.html","text","html",None);
        test_classification("text_html_head_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_script_20() {
        test_classification("text_html_script_20.html","text","html",None);
        test_classification("text_html_script_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_script_3e() {
        test_classification("text_html_script_3e.html","text","html",None);
        test_classification("text_html_script_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_iframe_20() {
        test_classification("text_html_iframe_20.html","text","html",None);
        test_classification("text_html_iframe_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_iframe_3e() {
        test_classification("text_html_iframe_3e.html","text","html",None);
        test_classification("text_html_iframe_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_h1_20() {
        test_classification("text_html_h1_20.html","text","html",None);
        test_classification("text_html_h1_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_h1_3e() {
        test_classification("text_html_h1_3e.html","text","html",None);
        test_classification("text_html_h1_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_div_20() {
        test_classification("text_html_div_20.html","text","html",None);
        test_classification("text_html_div_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_div_3e() {
        test_classification("text_html_div_3e.html","text","html",None);
        test_classification("text_html_div_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_font_20() {
        test_classification("text_html_font_20.html","text","html",None);
        test_classification("text_html_font_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_font_3e() {
        test_classification("text_html_font_3e.html","text","html",None);
        test_classification("text_html_font_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_table_20() {
        test_classification("text_html_table_20.html","text","html",None);
        test_classification("text_html_table_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_table_3e() {
        test_classification("text_html_table_3e.html","text","html",None);
        test_classification("text_html_table_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_a_20() {
        test_classification("text_html_a_20.html","text","html",None);
        test_classification("text_html_a_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_a_3e() {
        test_classification("text_html_a_3e.html","text","html",None);
        test_classification("text_html_a_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_style_20() {
        test_classification("text_html_style_20.html","text","html",None);
        test_classification("text_html_style_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_style_3e() {
        test_classification("text_html_style_3e.html","text","html",None);
        test_classification("text_html_style_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_title_20() {
        test_classification("text_html_title_20.html","text","html",None);
        test_classification("text_html_title_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_title_3e() {
        test_classification("text_html_title_3e.html","text","html",None);
        test_classification("text_html_title_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_b_20() {
        test_classification("text_html_b_20.html","text","html",None);
        test_classification("text_html_b_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_b_3e() {
        test_classification("text_html_b_3e.html","text","html",None);
        test_classification("text_html_b_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_body_20() {
        test_classification("text_html_body_20.html","text","html",None);
        test_classification("text_html_body_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_body_3e() {
        test_classification("text_html_body_3e.html","text","html",None);
        test_classification("text_html_body_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_br_20() {
        test_classification("text_html_br_20.html","text","html",None);
        test_classification("text_html_br_20_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_br_3e() {
        test_classification("text_html_br_3e.html","text","html",None);
        test_classification("text_html_br_3e_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_p_20() {
        test_classification("text_html_p_20.html","text","html",None);
        test_classification("text_html_p_20_u.html","text","html",None);
    }
    #[test]
    fn test_text_html_p_3e() {
        test_classification("text_html_p_3e.html","text","html",None);
        test_classification("text_html_p_3e_u.html","text","html",None);
    }

    #[test]
    fn test_text_html_comment_20() {
        test_classification("text_html_comment_20.html","text","html",None);
    }

    #[test]
    fn test_text_html_comment_3e() {
        test_classification("text_html_comment_3e.html","text","html",None);
    }

    #[test]
    fn test_xml() {
        test_classification("test.xml","text","xml",None);
    }

    #[test]
    fn test_pdf() {
        test_classification("test.pdf","application","pdf",None);
    }

    #[test]
    fn test_postscript() {
        test_classification("test.ps","application","postscript",None);
    }

    #[test]
    fn test_utf_16be_bom() {
        test_classification("utf16bebom.txt","text","plain",None);
    }

    #[test]
    fn test_utf_16le_bom() {
        test_classification("utf16lebom.txt","text","plain",None);
    }

    #[test]
    fn test_utf_8_bom() {
        test_classification("utf8bom.txt","text","plain",None);
    }

    #[test]
    fn test_rss_feed() {
        test_classification_full(&Path::new("text/xml/feed.rss"),"application","rss+xml",Some(("text","html")));
    }

    #[test]
    fn test_atom_feed() {
        test_classification_full(&Path::new("text/xml/feed.atom"),"application","atom+xml",Some(("text","html")));
    }
}
