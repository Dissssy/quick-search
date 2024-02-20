use super::{SearchResult, Searchable};
use anyhow::Result;

#[derive(Copy, Clone)]
pub struct Dictionary;

impl Searchable for Dictionary {
    fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut res: Vec<SearchResult> = vec![];

        // attempt to get the definition of the query word

        // if let Some(def) = webster::dictionary(query) {
        //     res.push(SearchResult {
        //         name: def.to_owned(),
        //         context: None,
        //         source: Box::new(*self),
        //         action: Box::new(move || {
        //             // copy result to clipboard
        //             super::set_clipboard(def);
        //         }),
        //     });
        // }

        match DictionaryApiResponse::get_word(query) {
            Ok(response) => {
                for response in response {
                    for meaning in response.meanings {
                        for definition in meaning.definitions {
                            let pos = meaning.part_of_speech.to_string();
                            let resstr = format!("{}: {}", pos, definition.definition);

                            res.push(SearchResult {
                                source: Box::new(*self),
                                name: resstr.clone(),
                                context: definition.example.clone(),
                                action: Some(Box::new({
                                    let definition = definition.clone();
                                    move || {
                                        // copy result to clipboard
                                        super::set_clipboard(&format!(
                                            "{}: {}{}",
                                            pos,
                                            definition.definition,
                                            definition.example.clone().map(|x| format!("\n{}", x)).unwrap_or_default()
                                        ));
                                    }
                                })),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        };

        // res.sort_by(|a, b| a.name.cmp(&b.name));
        // res.dedup_by(|a, b| a.name == b.name);

        res
    }
    fn copy(&self) -> Box<dyn Searchable> {
        Box::new(*self)
    }
    fn name(&self) -> String {
        "Definition".to_string()
    }
    fn priority(&self) -> i32 {
        900
    }
    // fn color(&self) -> Option<egui::Color32> {
    //     Some(egui::Color32::from_rgb(0, 255, 0))
    // }

    fn colored_name(&self) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            &self.name(),
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(0, 255, 0),
                ..Default::default()
            },
        );
        job
    }
}

#[derive(serde::Deserialize, Debug)]
struct DictionaryApiResponse {
    // word: String,
    // phonetic: String,
    // phonetics: Vec<Phonetic>,
    meanings: Vec<Meaning>,
    // license: License,
    // #[serde(rename = "sourceUrls")]
    // source_urls: Vec<String>,
}

// #[derive(serde::Deserialize, Debug)]
// struct Phonetic {
//     text: String,
//     audio: String,
//     #[serde(rename = "sourceUrl")]
//     source_url: Option<String>,
//     license: Option<License>,
// }

#[derive(serde::Deserialize, Debug, Clone)]
struct Meaning {
    #[serde(rename = "partOfSpeech")]
    // part_of_speech: PartOfSpeech,
    part_of_speech: String,
    definitions: Vec<WordDefinition>,
    // synonyms: Vec<String>,
    // antonyms: Vec<String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct WordDefinition {
    definition: String,
    example: Option<String>,
    // synonyms: Vec<String>,
    // antonyms: Vec<String>,
}

// #[derive(serde::Deserialize, Debug)]
// struct License {
//     name: String,
//     url: String,
// }

// #[derive(serde::Deserialize, Debug, Clone)]
// enum PartOfSpeech {
//     #[serde(rename = "noun")]
//     Noun,
//     #[serde(rename = "verb")]
//     Verb,
//     #[serde(rename = "adjective")]
//     Adjective,
//     #[serde(rename = "adverb")]
//     Adverb,
//     #[serde(rename = "pronoun")]
//     Pronoun,
//     #[serde(rename = "preposition")]
//     Preposition,
//     #[serde(rename = "conjunction")]
//     Conjunction,
//     #[serde(rename = "determiner")]
//     Determiner,
//     #[serde(rename = "exclamation")]
//     Exclamation,
//     #[serde(rename = "interjection")]
//     Interjection,
// }

// impl std::fmt::Display for PartOfSpeech {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             PartOfSpeech::Noun => write!(f, "noun"),
//             PartOfSpeech::Verb => write!(f, "verb"),
//             PartOfSpeech::Adjective => write!(f, "adjective"),
//             PartOfSpeech::Adverb => write!(f, "adverb"),
//             PartOfSpeech::Pronoun => write!(f, "pronoun"),
//             PartOfSpeech::Preposition => write!(f, "preposition"),
//             PartOfSpeech::Conjunction => write!(f, "conjunction"),
//             PartOfSpeech::Determiner => write!(f, "determiner"),
//             PartOfSpeech::Exclamation => write!(f, "exclamation"),
//         }
//     }
// }

impl DictionaryApiResponse {
    fn get_word(word: &str) -> Result<Vec<Self>> {
        let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", urlencoding::encode(word));

        let client = reqwest::blocking::Client::new();

        let response = client.get(url).send()?;

        let json = response.json()?;

        Ok(json)
    }
}
