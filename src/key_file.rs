use super::error::Error;

use std::collections::HashMap;
use std::str;

use nom::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Locale<'a> {
    lang: &'a str,
    country: Option<&'a str>,
    modifier: Option<&'a str>,
}

impl<'a> Locale<'a> {
    fn new(locale: &str) -> Option<Locale> {
        if let IResult::Done(b"", locale) = locale_parser(locale.as_bytes()) {
            Some(locale)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Key<'a> {
    key: &'a str,
    locale: Option<Locale<'a>>,
}

impl<'a> Key<'a> {
    fn new(key: &'a str, locale: Option<Locale<'a>>) -> Key<'a> {
        Key {
            key: key,
            locale: locale,
        }
    }
}

named!(alpha_str(&[u8]) -> &str,
    map_res!(alpha, str::from_utf8)
);

named!(locale_parser(&[u8]) -> Locale,
    chain!(
        lang: alpha_str ~
        country: opt!(complete!(preceded!(tag!("_"), alpha_str))) ~
        modifier: opt!(complete!(preceded!(tag!("@"), alpha_str))) ,

        || Locale { lang: lang,
                    country: country,
                    modifier: modifier,
                  }
    )
);

named!(comment_parser(&[u8]) -> &[u8],
    chain!(
        space? ~
        tag!("#") ~
        text: not_line_ending ,

        || text
    )
);

fn is_key_char(chr: u8) -> bool {
    is_alphabetic(chr) || chr == b'-'
}

named!(key_value_parser(&[u8]) -> (Key, &str),
    chain!(
        many0!(alt!(comment_parser | multispace)) ~
        key: map_res!(take_while1!(is_key_char), str::from_utf8) ~
        locale: opt!(delimited!(tag!("["), locale_parser, tag!("]"))) ~
        opt!(space) ~
        tag!("=") ~
        value: map_res!(not_line_ending, str::from_utf8) ,

        || { (Key { key: key, locale: locale }, value) }
    )
);

named!(group_header_parser(&[u8]) -> &str,
    terminated!(
        delimited!(
            tag!("["),
            map_res!(take_until!("]"), |s| str::from_utf8(s)),
            tag!("]")
        ),
        opt!(multispace)
    )
);

named!(group_parser(&[u8]) -> (&str, HashMap<Key, &str>),
    chain!(
        name: group_header_parser ~
        kvs: many0!(key_value_parser) ~
        many0!(alt!(comment_parser | multispace)) ,

        || (name, kvs.into_iter().collect())
    )
);

named!(keyfile_parser(&[u8]) -> Vec<(&str, HashMap<Key, &str>)>,
    preceded!(
        many0!(alt!(comment_parser | multispace)),
        many0!(group_parser)
    )
);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyFile<'a>(HashMap<&'a str, HashMap<Key<'a>, &'a str>>);

impl<'a> KeyFile<'a> {
    pub fn read_bytes(buf: &[u8]) -> Result<KeyFile, Error> {
        if let IResult::Done(_, groups) = keyfile_parser(buf) {
            Ok(KeyFile(groups.into_iter().collect()))
        } else {
            Err(Error::Parse)
        }
    }

    fn get_value(&self, group_name: &str, key: &str, locale: Option<Locale>) -> Option<&str> {
        self.0
            .get(group_name)
            .and_then(|group| group.get(&Key::new(key, locale)).cloned())
    }

    pub fn get_default_string(&self, group_name: &str, key: &str) -> Option<&str> {
        self.get_value(group_name, key, None)
    }

    pub fn get_localized_string(&self, group_name: &str, key: &str, locale: &str) -> Option<&str> {
        Locale::new(locale).and_then(|l| self.get_value(group_name, key, Some(l)))
    }

    pub fn get_boolean(&self, group_name: &str, key: &str) -> Option<bool> {
        self.get_value(group_name, key, None).and_then(|value| {
            match value {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::KeyFile;

    #[test]
    fn test_keyfile() {
        let text = "
 # a comment

[Desktop Entry]
Name=Firefox
GenericName=Web Browser

# another comment
GenericName[ar]=متصفح ويب
Hidden=false
GenericName[ast]=Restolador Web
NoDisplay=true
GenericName[en_US@Mod]=Web Browser

[Second Section]
k=v
";
        let kf = KeyFile::read_bytes(text.as_bytes()).unwrap();

        assert_eq!(kf.get_default_string("Desktop Entry", "Name"),
                   Some("Firefox"));
        assert_eq!(kf.get_default_string("Desktop Entry", "GenericName"),
                   Some("Web Browser"));
        assert_eq!(kf.get_default_string("Second Section", "k"), Some("v"));

        assert_eq!(kf.get_localized_string("Desktop Entry", "GenericName", "ast"),
                   Some("Restolador Web"));
        assert_eq!(kf.get_localized_string("Desktop Entry", "GenericName", "en_US@Mod"),
                   Some("Web Browser"));

        assert_eq!(kf.get_default_string("Desktop Entry", "Blah"), None);
        assert_eq!(kf.get_default_string("DesktopEntry", "Name"), None);
        assert_eq!(kf.get_localized_string("Desktop Entry", "GenericName", "foo"),
                   None);

        assert_eq!(kf.get_boolean("Desktop Entry", "Hidden"), Some(false));
        assert_eq!(kf.get_boolean("Desktop Entry", "NoDisplay"), Some(true));
        assert_eq!(kf.get_boolean("Desktop Entry", "Name"), None);
    }
}
