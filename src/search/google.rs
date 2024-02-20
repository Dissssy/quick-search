use super::{SearchResult, Searchable};

#[derive(Copy, Clone)]
pub struct Google;

impl Google {
    fn get_individual_query(&self, query: String) -> SearchResult {
        SearchResult {
            source: Box::new(*self),
            name: query.clone(),
            context: None,
            action: Some(Box::new(move || {
                // open in browser
                if let Err(e) = webbrowser::open(&format!("https://google.com/search?q={}", urlencoding::encode(&query))) {
                    println!("failed to open browser: {}", e);
                }
            })),
        }
    }
}

impl Searchable for Google {
    fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut res: Vec<SearchResult> = vec![self.get_individual_query(query.to_owned())];

        // google search autocomplete, this is in its own thread so we can block
        // URL = https://google.com/complete/search?q=ytest&output=toolbar&hl=en

        let url = format!("https://google.com/complete/search?q={}&output=toolbar&hl=en", urlencoding::encode(query));

        let client = reqwest::blocking::Client::new();

        if let Ok(response) = client.get(url).send() {
            // example data i copied to help copilot:
            // <toplevel>
            //  <CompleteSuggestion>
            //   <suggestion data="test"/>
            //  </CompleteSuggestion>
            //  <CompleteSuggestion>
            //   <suggestion data="testosterone"/>
            //  </CompleteSuggestion>
            //  <CompleteSuggestion>
            //   <suggestion data="test my internet speed"/>
            //  </CompleteSuggestion>
            //  <CompleteSuggestion>
            //   <suggestion data="test internet speed"/>
            //  </CompleteSuggestion>
            //  <CompleteSuggestion>
            //   <suggestion data="testicular torsion"/>
            //  </CompleteSuggestion>
            //  <CompleteSuggestion>
            //   <suggestion data="testicular cancer"/>
            //  </CompleteSuggestion>
            //  <CompleteSuggestion>
            //   <suggestion data="testosterone booster"/>
            //  </CompleteSuggestion>
            //  <script/>
            // </toplevel>

            if let Ok(text) = response.text() {
                if let Ok(xml) = roxmltree::Document::parse(&text) {
                    for node in xml.descendants() {
                        if node.tag_name().name() == "suggestion" {
                            if let Some(data) = node.attribute("data") {
                                let data = data.to_string();
                                res.push(self.get_individual_query(data));
                            }
                        }
                    }
                }
            }
        }

        // res.sort_by(|a, b| a.name.cmp(&b.name));
        res.dedup_by(|a, b| a.name == b.name);

        res
    }

    fn copy(&self) -> Box<dyn Searchable> {
        Box::new(*self)
    }
    fn name(&self) -> String {
        "Google".to_string()
    }
    fn priority(&self) -> i32 {
        0
    }
    // fn color(&self) -> Option<egui::Color32> {
    //     Some(egui::Color32::from_rgb(0, 255, 0))
    // }
    fn colored_name(&self) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            "G",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(66, 133, 244),
                ..Default::default()
            },
        );
        job.append(
            "o",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(219, 68, 55),
                ..Default::default()
            },
        );
        job.append(
            "o",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(244, 180, 0),
                ..Default::default()
            },
        );
        job.append(
            "g",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(66, 133, 244),
                ..Default::default()
            },
        );
        job.append(
            "l",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(15, 157, 88),
                ..Default::default()
            },
        );
        job.append(
            "e",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(219, 68, 55),
                ..Default::default()
            },
        );
        job
    }
}
