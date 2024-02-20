use mexprp::{ParseError, Term};

use super::{SearchResult, Searchable};

#[derive(Copy, Clone)]
pub struct Math;

impl Searchable for Math {
    fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut res = vec![];

        // attempt to parse query as a mathematical expression

        if let Ok::<Term<f64>, ParseError>(term) = mexprp::Term::parse(query) {
            // if it parses, then we can evaluate it
            match term.eval() {
                Ok(result) => {
                    let result = match result {
                        mexprp::Answer::Single(x) => vec![x],
                        mexprp::Answer::Multiple(x) => x,
                    };

                    for result in result {
                        let resstr = format!("{} = {}", term, result);

                        res.push(SearchResult {
                            source: Box::new(*self),
                            name: resstr.clone(),
                            context: None,
                            action: Some(Box::new(move || {
                                // copy result to clipboard
                                super::set_clipboard(&result.to_string());
                            })),
                        });
                    }

                    // let resstr = format!("{} = {}", term, result);

                    // res.push(SearchResult {
                    //     name: resstr.clone(),
                    //     action: Box::new(move || {
                    //         // copy result to clipboard
                    //         println!("copied to clipboard: {}", result);
                    //     }),
                    // });
                }
                Err(_e) => {
                    // println!("error: {}", e);
                }
            };
        }

        res.sort_by(|a, b| a.name.cmp(&b.name));
        res.dedup_by(|a, b| a.name == b.name);

        res
    }
    fn copy(&self) -> Box<dyn Searchable> {
        Box::new(*self)
    }
    fn name(&self) -> String {
        "Math".to_string()
    }
    fn priority(&self) -> i32 {
        1000
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
                color: egui::Color32::from_rgb(22, 190, 47),
                ..Default::default()
            },
        );
        job
    }
}
