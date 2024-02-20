use super::{SearchResult, Searchable};
use anyhow::Result;

#[derive(Copy, Clone)]
pub struct UrbanDictionary;

impl Searchable for UrbanDictionary {
    fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut res = vec![];

        match DictionaryApiResponse::get_word(query) {
            Ok(response) => {
                for meaning in response.list {
                    let resstr = meaning.definition.clone();

                    res.push((
                        SearchResult {
                            source: Box::new(*self),
                            name: resstr.clone(),
                            context: Some(meaning.example.clone()),
                            action: Some(Box::new({
                                let definition = meaning.clone();
                                move || {
                                    // copy result to clipboard
                                    super::set_clipboard(&format!("{} | {}", definition.definition, definition.example.clone()));
                                }
                            })),
                        },
                        meaning.thumbs_up - meaning.thumbs_down,
                    ));
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        };

        res.sort_by(|a, b| a.0.name.cmp(&b.0.name));
        res.dedup_by(|a, b| a.0.name == b.0.name);
        res.sort_by(|a, b| a.1.cmp(&b.1));

        res.into_iter().map(|x| x.0).collect()
    }
    fn copy(&self) -> Box<dyn Searchable> {
        Box::new(*self)
    }
    fn name(&self) -> String {
        "UrbanDictionary".to_string()
    }
    fn priority(&self) -> i32 {
        850
    }
    // fn color(&self) -> Option<egui::Color32> {
    //     Some(egui::Color32::from_rgb(0, 255, 0))
    // }

    fn colored_name(&self) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            "Urban",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(228, 247, 20),
                ..Default::default()
            },
        );
        job.append(
            "Dictionary",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(17, 78, 232),
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
    list: Vec<Meaning>,
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
    definition: String,
    // permalink: String,
    thumbs_up: i32,
    // author: String,
    // word: String,
    // defid: i32,
    // current_vote: String,
    // written_on: String,
    example: String,
    thumbs_down: i32,
}

impl DictionaryApiResponse {
    fn get_word(word: &str) -> Result<Self> {
        let url = format!("https://api.urbandictionary.com/v0/define?term={}", urlencoding::encode(word));

        let client = reqwest::blocking::Client::new();

        let response = client.get(url).send()?;

        let mut json: Self = response.json()?;

        json.list.iter_mut().for_each(|meaning| {
            meaning.definition = meaning.definition.replace(['[', ']', '\r', '\n'], "");
            meaning.example = meaning.example.replace(['[', ']', '\r', '\n'], "");
        });

        Ok(json)
    }
}
