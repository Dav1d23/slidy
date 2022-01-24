use slidy::windows::slideshow::{
    Section, SectionFigure, SectionMain, SectionText, Slide, Slideshow, Vec2,
};

pub fn prepare_slides(rot: f32, text: String, c1: u8, c2: u8) -> Slideshow {
    Slideshow {
        slides: vec![{
            Slide {
                bg_color: Some((c1, 12, c2, 255).into()),
                sections: vec![
                    Section {
                        size: Some(Vec2 { x: 0.04, y: 0.08 }),
                        position: Some(Vec2 { x: 0.1, y: 0.1 }),
                        sec_main: Some(SectionMain::Text(SectionText {
                            text,
                            color: Some((c1, 255 - c2, 100, 255).into()),
                            font: None,
                        })),
                    },
                    Section {
                        size: Some(Vec2 { x: 0.3, y: 0.3 }),
                        position: Some(Vec2 { x: 0.2, y: 0.3 }),
                        sec_main: Some(SectionMain::Figure(SectionFigure {
                            path: String::from("resources/star.jpg"),
                            rotation: rot,
                        })),
                    },
                    Section {
                        size: Some(Vec2 { x: 0.2, y: 0.2 }),
                        position: Some(Vec2 { x: 0.6, y: 0.6 }),
                        sec_main: Some(SectionMain::Figure(SectionFigure {
                            path: String::from("resources/star.jpg"),
                            rotation: -rot + 369.3,
                        })),
                    },
                ],
            }
        }],
        ..Default::default()
    }
}
