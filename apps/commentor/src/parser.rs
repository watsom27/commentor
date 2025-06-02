use std::{fs::File, io::Read, path::Path, rc::Rc};

use logos::Logos;

pub struct ConfigFile {
    pub editor_command: String,
    pub github_token:   String,
    pub pr_url:         String,
    pub comments:       Rc<[String]>,
}

#[derive(Logos, Debug, PartialEq)]
pub enum Token<'a> {
    #[regex("(http(s)?:\\/\\/)?[A-Za-z0-9-_\\.\\/]+")]
    Identifier(&'a str),

    #[token(":")]
    Colon,

    #[token("\n")]
    NewLine,

    #[token(" ")]
    Space,

    #[regex("#{1}.*")]
    Comment,
}

pub fn read_config_file(path: &Path) -> anyhow::Result<ConfigFile> {
    let mut file = File::open(path)?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;

    let mut lex = Token::lexer(&source).peekable();

    let mut editor_command = String::new();
    let mut github_token = String::new();
    let mut pr_url = String::new();
    let mut comments = vec![];

    macro_rules! matched {
        ($lex:ident, $token:pat) => {
            $lex.next_if(|t| matches!(t.as_ref().unwrap(), $token))
        };
    }

    macro_rules! expect {
        ($lex:ident, $token:pat) => {
            $lex.next_if(|t| matches!(t.as_ref().unwrap(), $token))
                .expect(&format!(
                    "Expected token {:?}, but got something else...",
                    stringify!($token)
                ))
                .unwrap()
        };
    }

    if matched!(lex, Token::Identifier("editor")).is_some() {
        expect!(lex, Token::Colon);
        matched!(lex, Token::Space); // Optional space
        let ec = expect!(lex, Token::Identifier(_));
        expect!(lex, Token::NewLine);

        log::trace!("editor command: {ec:?}");
        if let Token::Identifier(ec) = ec {
            editor_command = ec.to_owned();
        } else {
            unreachable!()
        }
    }

    if matched!(lex, Token::Identifier("github_token")).is_some() {
        expect!(lex, Token::Colon);
        matched!(lex, Token::Space);
        let gt = expect!(lex, Token::Identifier(_));
        expect!(lex, Token::NewLine);

        log::trace!("github_token: {gt:?}");
        if let Token::Identifier(gt) = gt {
            github_token = gt.to_owned();
        } else {
            unreachable!()
        }
    }

    if matched!(lex, Token::Identifier("pr_url")).is_some() {
        expect!(lex, Token::Colon);
        matched!(lex, Token::Space);
        let pl = expect!(lex, Token::Identifier(_));
        expect!(lex, Token::NewLine);

        log::trace!("pr_link: {pl:?}");
        if let Token::Identifier(pl) = pl {
            pr_url = pl.to_owned();
        } else {
            unreachable!()
        }
    }

    if matched!(lex, Token::Identifier("comments")).is_some() {
        expect!(lex, Token::Colon);
        expect!(lex, Token::NewLine);

        while lex.peek().is_some() {
            if lex
                .next_if(|t| matches!(t.as_ref().unwrap(), Token::Comment))
                .is_some()
            {
                while lex
                    .next_if(|t| matches!(t.as_ref().unwrap(), Token::NewLine))
                    .is_none()
                {
                    log::trace!("Skipping comments...");
                }
            }

            if let Some(comment) =
                lex.next_if(|t| matches!(t.as_ref().unwrap(), Token::Identifier(_)))
            {
                log::trace!("found command(comment): {comment:?}");
                if let Token::Identifier(comment) = comment.unwrap() {
                    comments.push(comment.to_owned());
                } else {
                    unreachable!()
                }

                expect!(lex, Token::NewLine);
            }
        }
    }

    Ok(ConfigFile {
        editor_command,
        github_token,
        pr_url,
        comments: comments.into(),
    })
}
