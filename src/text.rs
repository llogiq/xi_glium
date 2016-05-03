use std::collections::BTreeMap;
use serde_json::Value;

use renderer::*;

const LINE_HEIGHT: f32 = 20.;
const LEFT_MARGIN: f32 = 15.;

// #[derive(Clone)]
pub struct Line<'a> {
    pub text: String,
    pub cursor: Option<u64>,
    pub renderer: LineRenderer<'a>, // This is the lifetime that infects the hierarchy up to State
}

impl<'a> Line<'a> {

    pub fn placeholder(renderer: &'a Renderer) -> Line<'a> {
        let text = ">>> NOT IN CACHE <<<";
        let renderer = LineRenderer::new(renderer, text);
        Line { text: text.into(), cursor: None, renderer: renderer }
    }
}

pub struct Text<'a> {
    cache: BTreeMap<u64, Line<'a>>,
    placeholder_line: Line<'a>,
    pub top: f64,
    pub height: f64,
    n_lines: u64,
    renderer: TextRenderer,
    // scrollbar: Primitive,
}

impl<'a> Text<'a> {
    pub fn new(renderer: &Renderer) -> Text {
        Text {
            cache: BTreeMap::new(),
            placeholder_line: Line::placeholder(renderer),
            top: 0.,
            height: 0.,
            n_lines: 0,
            renderer: TextRenderer::new(renderer, LEFT_MARGIN)
        }
    }

    pub fn refresh(&mut self, n_lines: u64) {
        self.cache.clear();
        self.n_lines = n_lines;
    }

    pub fn render(&self, target: &mut Target) {
        self.renderer.draw(target, &self.get_lines());
    }

    pub fn add_lines(&mut self, renderer: &'a Renderer, value: &Value, first: u64) {
        for (i, line) in value.as_array().unwrap().into_iter().enumerate() {
            let line = line.as_array().unwrap();
            let text = line[0].as_string().unwrap().to_string();
            // annotations
            let mut cursor = None;
            for annotation in line.iter().skip(1).map(|a| a.as_array().unwrap()) {
                match annotation[0].as_string().unwrap() {
                    "cursor" => {
                        cursor = Some(annotation[1].as_u64().unwrap());
                    },
                    _ => () // ignore unknown annotations
                }
            }
            let renderer = LineRenderer::new(renderer, &text);
            self.cache.insert(i as u64+first, Line { text: text, cursor: cursor, renderer: renderer });
        }
    }

    pub fn scroll_to(&mut self, line: u64, _column: u64) {
        let min = |a,b| if a > b { b } else { a };
        let max = |a,b| if a < b { b } else { a };
        self.top = max(0., min(self.top, line as f64 - 2.)); // scroll up
        self.top = min(self.n_lines as f64, max(self.top, line as f64 - self.height + 1. + 2.)); // scroll dn
    }

    pub fn get_lines(&self) -> Vec<(f32, &Line)> {
        self.get_line_pos().into_iter().filter_map(|(pos,i)| self.get_line(i).map(|x| (pos,x))).collect()
    }

    // Return: Vec<(line_pos, line_id)>
    pub fn get_line_pos(&self) -> Vec<(f32, u64)> {
        (self.top as u64 .. (self.top + self.height).ceil() as u64)
            .map(|i| ((self.height - i as f64 + self.top - 0.5) as f32 * LINE_HEIGHT, i)
            ).collect()
    }

    pub fn get_line_col(&self, px: i32, py: i32) -> (u64,u64) {
        let line = self.get_line_pos().into_iter().min_by_key(|&(y,_)| (y as i32 - py).abs()).unwrap().1;
        let column = if let Some(line) = self.get_line(line) {
            line.renderer.char_pos_x.iter().enumerate().min_by_key(|&(_,x)| {
                (*x as i32 - px + LEFT_MARGIN as i32).abs()
            }).unwrap().0 as u64
        } else { // after the text
            0
        };
        (line, column)
    }

    fn get_line(&self, n: u64) -> Option<&Line> {
        if n >= self.n_lines {
            None
        } else {
            Some(self.cache.get(&n).unwrap_or(&self.placeholder_line))
        }
    }

    pub fn set_size(&mut self, _w: u32, h: u32) {
        self.height = h as f64 / LINE_HEIGHT as f64;
    }
}
