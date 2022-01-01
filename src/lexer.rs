use alloc::borrow::Cow;
use alloc::collections::VecDeque;
use alloc::string::ToString;
use alloc::vec::Vec;

enum LexerState {
    /// Reading ArtistName, ends if special characters were found
    ///
    /// Special character list
    /// - Escape character: '\'
    /// - Left bracket: '（'
    /// - Right bracket: '）'
    /// - Comma: '、'
    ///
    /// When escape character is found, state goes to EscapeStart
    Normal,
    /// Reading escaped character after escape character '\'
    EscapeStart,
}

pub(crate) struct Tokens<'input> {
    inner: VecDeque<Token<'input>>,
}

impl<'input> Tokens<'input> {
    pub(crate) fn parse(input: &'input str) -> Self {
        let mut tokens = VecDeque::new();
        let mut state = LexerState::Normal;

        let mut image_start = 0;
        let mut image_size = 0;
        let mut image_owned = None;

        let chars: Vec<_> = input.chars().collect();
        let mut i = 0;
        loop {
            if i >= chars.len() {
                break;
            }

            let ch = chars[i];
            match state {
                LexerState::Normal => {
                    if ch == '\\' || (ch == '、' && chars.len() > i + 1 && chars[i + 1] == '、') {
                        state = LexerState::EscapeStart;
                        if let None = image_owned {
                            image_owned = Some(input[image_start..image_start + image_size].to_string());
                        }
                        image_size += ch.len_utf8();
                    } else if ch == '（' || ch == '）' || ch == '、' {
                        // push previous ArtistName if exists
                        if image_size > 0 {
                            let image = if let Some(image) = image_owned {
                                image_owned = None;
                                Cow::Owned(image)
                            } else {
                                Cow::Borrowed(&input[image_start..image_start + image_size])
                            };
                            tokens.push_back(Token {
                                token_type: TokenType::ArtistName,
                                image,
                            });
                            image_start += image_size;
                        }

                        image_size = ch.len_utf8();
                        let token_type = if ch == '（' {
                            TokenType::LBracket
                        } else if ch == '）' {
                            TokenType::RBracket
                        } else /* ch == '、' */ {
                            TokenType::Comma
                        };
                        tokens.push_back(Token {
                            token_type,
                            image: Cow::Borrowed(&input[image_start..image_start + image_size]),
                        });
                        image_start += image_size;
                        image_size = 0;
                    } else {
                        if let Some(ref mut image) = image_owned {
                            image.push(ch);
                        }
                        image_size += ch.len_utf8();
                    }
                    i += 1;
                }
                LexerState::EscapeStart => {
                    state = LexerState::Normal;
                    if let Some(ref mut image) = image_owned {
                        image.push(ch);
                    }
                    image_size += ch.len_utf8();
                    i += 1;
                }
            }
        }

        // push the final artist
        if image_size > 0 {
            let image = if let Some(image) = image_owned {
                Cow::Owned(image)
            } else {
                Cow::Borrowed(&input[image_start..image_start + image_size])
            };
            tokens.push_back(Token {
                token_type: TokenType::ArtistName,
                image,
            });
        }

        Self {
            inner: tokens,
        }
    }

    pub(crate) fn into_inner(self) -> VecDeque<Token<'input>> {
        self.inner
    }
}

pub(crate) struct Token<'input> {
    token_type: TokenType,
    image: Cow<'input, str>,
}

impl<'input> Token<'input> {
    pub(crate) fn is_artist_name(&self) -> bool {
        match self.token_type {
            TokenType::ArtistName => true,
            _ => false,
        }
    }

    pub(crate) fn is_left_bracket(&self) -> bool {
        match self.token_type {
            TokenType::LBracket => true,
            _ => false,
        }
    }

    pub(crate) fn is_right_bracket(&self) -> bool {
        match self.token_type {
            TokenType::RBracket => true,
            _ => false,
        }
    }

    pub(crate) fn is_comma(&self) -> bool {
        match self.token_type {
            TokenType::Comma => true,
            _ => false,
        }
    }

    pub(crate) fn into_inner(self) -> Cow<'input, str> {
        self.image
    }
}

enum TokenType {
    ArtistName,
    LBracket,
    RBracket,
    Comma,
}

#[cfg(test)]
mod tests {
    use crate::lexer::Tokens;

    #[test]
    fn lexer_input_normal() {
        let result = Tokens::parse("Petit Rabbit's（ココア（佐倉綾音）、チノ（水瀬いのり）、リゼ（種田梨沙）、千夜（佐藤聡美）、シャロ（内田真礼））、Append、、、、、");
        assert_eq!(result.inner.len(), 30);
        assert_eq!(result.inner[0].image, "Petit Rabbit's");
        assert_eq!(result.inner[2].image, "ココア");
        assert_eq!(result.inner[4].image, "佐倉綾音");
        assert_eq!(result.inner[7].image, "チノ");
        assert_eq!(result.inner[9].image, "水瀬いのり");
        assert_eq!(result.inner[12].image, "リゼ");
        assert_eq!(result.inner[14].image, "種田梨沙");
        assert_eq!(result.inner[17].image, "千夜");
        assert_eq!(result.inner[19].image, "佐藤聡美");
        assert_eq!(result.inner[22].image, "シャロ");
        assert_eq!(result.inner[24].image, "内田真礼");
        assert_eq!(result.inner[28].image, "Append、、");
        assert_eq!(result.inner[29].image, "、");
    }

    #[test]
    fn lexer_input_escape() {
        let result = Tokens::parse("Petit\\、Rabbit's（ココア（佐倉綾音）、チノ（水瀬いのり）、リゼ（種田梨沙）、千夜（佐藤聡美）、シャロ（内田真礼））");
        assert_eq!(result.inner.len(), 27);
        assert_eq!(result.inner[0].image, "Petit、Rabbit's");
        assert_eq!(result.inner[2].image, "ココア");
        assert_eq!(result.inner[4].image, "佐倉綾音");
        assert_eq!(result.inner[7].image, "チノ");
        assert_eq!(result.inner[9].image, "水瀬いのり");
        assert_eq!(result.inner[12].image, "リゼ");
        assert_eq!(result.inner[14].image, "種田梨沙");
        assert_eq!(result.inner[17].image, "千夜");
        assert_eq!(result.inner[19].image, "佐藤聡美");
        assert_eq!(result.inner[22].image, "シャロ");
        assert_eq!(result.inner[24].image, "内田真礼");
    }

    #[test]
    fn lexer_input_escape2() {
        let result = Tokens::parse("Petit \\\\Rabbit's（ココア（佐倉綾音）、チノ（水瀬いのり）、リゼ（種田梨沙）、千夜（佐藤聡美）、シャロ（内田真礼））");
        assert_eq!(result.inner.len(), 27);
        assert_eq!(result.inner[0].image, "Petit \\Rabbit's");
        assert_eq!(result.inner[2].image, "ココア");
        assert_eq!(result.inner[4].image, "佐倉綾音");
        assert_eq!(result.inner[7].image, "チノ");
        assert_eq!(result.inner[9].image, "水瀬いのり");
        assert_eq!(result.inner[12].image, "リゼ");
        assert_eq!(result.inner[14].image, "種田梨沙");
        assert_eq!(result.inner[17].image, "千夜");
        assert_eq!(result.inner[19].image, "佐藤聡美");
        assert_eq!(result.inner[22].image, "シャロ");
        assert_eq!(result.inner[24].image, "内田真礼");
    }

    #[test]
    fn lexer_input_escape3() {
        let result = Tokens::parse("Petit \\Rabbit's（ココア（佐倉綾音）、チノ（水瀬いのり）、リゼ（種田梨沙）、千夜（佐藤聡美）、シャロ（内田真礼））");
        assert_eq!(result.inner.len(), 27);
        assert_eq!(result.inner[0].image, "Petit Rabbit's");
        assert_eq!(result.inner[2].image, "ココア");
        assert_eq!(result.inner[4].image, "佐倉綾音");
        assert_eq!(result.inner[7].image, "チノ");
        assert_eq!(result.inner[9].image, "水瀬いのり");
        assert_eq!(result.inner[12].image, "リゼ");
        assert_eq!(result.inner[14].image, "種田梨沙");
        assert_eq!(result.inner[17].image, "千夜");
        assert_eq!(result.inner[19].image, "佐藤聡美");
        assert_eq!(result.inner[22].image, "シャロ");
        assert_eq!(result.inner[24].image, "内田真礼");
    }
}