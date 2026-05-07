use std::{error::Error, sync::Arc};

use eframe::egui::{self, Color32, Pos2, Stroke};
use musicxml::{
    datatypes::Step,
    elements::{AudibleType, MeasureElement, NoteType, PartElement},
    read_score_partwise,
};

pub fn display_xml(ui: &mut egui::Ui) -> Result<(), Box<dyn Error>> {
    let rect = ui.max_rect();
    let center = rect.center();
    match read_score_partwise("score.xml") {
        Ok(score) => {
            let mut measure_count = 0;
            for part_element in &score.content.part[0].content {
                match part_element {
                    PartElement::Measure(measure) => {
                        draw_measure(
                            ui,
                            &measure_count,
                            egui::pos2(20.0, (center.y - 10.0) / 2.0),
                            200.0,
                            &measure.content,
                        );
                        measure_count += 1;
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    Ok(())
}

pub fn load_font(ctx: &eframe::egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let fontd = Arc::new(egui::FontData::from_static(include_bytes!(
        "../../assets/Bravura.otf"
    )));
    fonts.font_data.insert("Bravura".to_owned(), fontd);

    fonts.families.insert(
        egui::FontFamily::Name("Music".into()),
        vec!["Bravura".to_owned()],
    );

    ctx.set_fonts(fonts);
}

// helpers

fn draw_measure(
    ui: &mut egui::Ui,
    measure_count: &usize,
    top_left: Pos2,
    width: f32,
    measure_content: &Vec<MeasureElement>,
) {
    // staff drawing
    let painter = ui.painter();

    let line_spacing = 10.0;
    let stroke: Stroke = Stroke::new(1.5, Color32::BLACK);

    let mut staves_num = 2;
    // for element in measure_content {
    //     if let MeasureElement::Attributes(attr) = element {
    //         if let Some(s) = &attr.content.staves {
    //             staves_num = *s.content.deref() as usize;
    //         }
    //     }
    // }

    let staff_height = 4.0 * line_spacing;
    let staff_spacing = 20.0;
    let staff_gap = 40.0;
    let full_staff_height = staves_num as f32 * staff_height + (staves_num as f32 * staff_gap);
    let measures_per_line = 5;

    let line_index = measure_count / measures_per_line;
    let measure_on_line = measure_count % measures_per_line;

    let x_start = top_left.x + (measure_on_line as f32 * width);
    let x_end = x_start + width;
    let y_offset = top_left.y + (line_index as f32 * (full_staff_height + staff_spacing));

    for stave in 0..staves_num {
        let stave_y = y_offset + stave as f32 * (staff_height + staff_gap);
        for line in 0..5 {
            let y = stave_y + (line as f32 * line_spacing);

            let start = Pos2::new(x_start, y);
            let end = Pos2::new(x_end, y);
            painter.line_segment([start, end], stroke);
        }

        painter.line_segment(
            [
                Pos2::new(x_end, stave_y),
                Pos2::new(x_end, stave_y + staff_height),
            ],
            stroke,
        );
    }

    // note drawing logic
    let mut note_x = 15.0 + x_start;
    for measure_element in measure_content {
        if let MeasureElement::Note(note) = measure_element {
            let mut note_y = y_offset;
            let c_pos = y_offset + 50.0;
            if let NoteType::Normal(normal) = &note.content.info {
                if let AudibleType::Pitch(pitch) = &normal.audible {
                    let step_val = match pitch.content.step.content {
                        Step::C => 0,
                        Step::D => 1,
                        Step::E => 2,
                        Step::F => 3,
                        Step::G => 4,
                        Step::A => 5,
                        Step::B => 6,
                    };
                    let octave_calc = (*pitch.content.octave.content as f32 - 4.0) * 7.0;
                    let steps = step_val as f32 + octave_calc;
                    let pix_offset = steps * 5.0;
                    // changes location based on octave
                    note_y = c_pos - pix_offset;
                }
            }
            let music_font = egui::FontId::new(40.0, egui::FontFamily::Name("Music".into()));
            painter.text(
                Pos2::new(note_x, note_y),
                egui::Align2::CENTER_CENTER,
                "\u{E0A4}",
                music_font,
                egui::Color32::BLACK,
            );
            note_x += 8.0;
        }
    }
}
