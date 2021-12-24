use alloc::borrow::Cow;
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::lexer::{Token, Tokens};

pub struct ArtistList<'input> {
    pub artists: Vec<Artist<'input>>,
}

impl<'input> ArtistList<'input> {
    pub fn parse(input: &'input str) -> Result<ArtistList, String> {
        let mut tokens = Tokens::parse(input).into_inner();
        ArtistList::parse_inner(&mut tokens)
    }

    pub fn len(&self) -> usize {
        self.artists.len()
    }

    // ArtistList ::= ArtistName ('（' ArtistList '）')? ('、' ArtistList)?
    fn parse_inner(tokens: &mut VecDeque<Token<'input>>) -> Result<ArtistList<'input>, String> {
        let mut result = ArtistList { artists: Vec::new() };

        let artist_name = tokens.pop_front().ok_or("Insufficient tokens".to_string())?;
        if !artist_name.is_artist_name() {
            return Err("Expected ArtistName".to_string());
        }

        if let Some(mut next) = tokens.pop_front() {
            if next.is_artist_name() || next.is_right_bracket() {
                // no artist list
                result.artists.push(Artist { name: artist_name.into_inner(), references: None });
                // push the next token back
                tokens.push_front(next);
            } else {
                if next.is_left_bracket() {
                    // nested artist list
                    let artist_list = ArtistList::parse_inner(tokens)?;
                    result.artists.push(Artist { name: artist_name.into_inner(), references: Some(artist_list) });

                    // RBracket
                    let rbracket = tokens.pop_front().expect("Insufficient token");
                    if !rbracket.is_right_bracket() {
                        return Err("Expected Right Bracket".to_string());
                    }

                    if let Some(next_next) = tokens.pop_front() {
                        next = next_next;
                    }
                }

                if next.is_comma() {
                    // appendix artist list
                    let mut artist_list = ArtistList::parse_inner(tokens)?;
                    result.artists.append(&mut artist_list.artists);
                    return Ok(result);
                } else {
                    tokens.push_front(next);
                }
            }
        } else {
            // token stream end
            result.artists.push(Artist { name: artist_name.into_inner(), references: None });
        }

        Ok(result)
    }
}

pub struct Artist<'input> {
    name: Cow<'input, str>,
    references: Option<ArtistList<'input>>,
}

impl<'input> Artist<'input> {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn references(&self) -> Option<&ArtistList> {
        self.references.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ArtistList;

    #[test]
    fn parser_input_normal() {
        let result = ArtistList::parse("Petit Rabbit's（ココア（佐倉綾音）、チノ（水瀬いのり）、リゼ（種田梨沙）、千夜（佐藤聡美）、シャロ（内田真礼））、Append")
            .expect("Failed to parse normal artist string");
        assert_eq!(result.len(), 2);

        assert_eq!(result.artists[0].name, "Petit Rabbit's");
        assert!(result.artists[0].references.is_some());
        // TODO: validate result

        assert_eq!(result.artists[1].name, "Append");
        assert!(result.artists[1].references.is_none());
    }
}