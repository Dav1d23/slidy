use slidy::slideshow::{
    Position, Section, SectionFigure, SectionMain, SectionText, Size, Slide,
    Slideshow,
};

pub fn prepare_slide(rot: f32, text: String, c1: u8, c2: u8) -> Slideshow {
    Slideshow {
        slides: vec![{
            Slide {
                bg_color: Some((c1, 12, c2, 255).into()),
                sections: vec![
                    Section {
                        size: Some(Size { w: 0.04, h: 0.08 }),
                        position: Some(Position { x: 0.1, y: 0.1 }),
                        sec_main: Some(SectionMain::Text(SectionText {
                            text,
                            color: Some((c1, 255 - c2, 100, 255).into()),
                            font: None,
                        })),
                    },
                    Section {
                        size: Some(Size { w: 0.3, h: 0.3 }),
                        position: Some(Position { x: 0.2, y: 0.3 }),
                        sec_main: Some(SectionMain::Figure(SectionFigure {
                            path: String::from("resources/star.jpg"),
                            rotation: rot,
                        })),
                    },
                    Section {
                        size: Some(Size { w: 0.2, h: 0.2 }),
                        position: Some(Position { x: 0.6, y: 0.6 }),
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
