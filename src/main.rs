mod input;
mod lexer;
mod parser;

use macroquad::prelude::*;
use miniquad::window::screen_size;

struct Vars {
    mid: (f32, f32),
    scale: f32, 
    step: f32, 
    font_size: f32, 
    iterations: i32,
    cam_mid: Vec2, 
    view: (Vec2, Vec2),
}

impl Vars {
    fn new(scale: f32) -> Self {
        Self {
            mid: (screen_width() / 2.0, screen_height() / 2.0),
            scale, 
            step: 50. * scale,
            font_size: 7. * scale,
            iterations: 30,
            cam_mid: vec2(0., 0.),
            view: (vec2(0., 0.), vec2(screen_width(), screen_height()))
        }
    }

    fn update(&mut self, scale: f32) {
        self.mid = (screen_width() / 2.0, screen_height() / 2.0);
        self.scale = scale;
        self.step = 50.0 * scale;
        self.font_size = 7.0 * scale;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
   
    let mut vars = Vars::new(1.0);

    //print read list
    let final_input = combine_numbers_and_chars(input::get_input().expect("function not correct"));
    println!("{:?}", final_input);

    //print lexed list 
    let input_lexed = lexer::tag(&final_input);
    lexer::print_tokens(&input_lexed);

    let tree = parser::TokenTree::parse_from_lexer(&input_lexed).unwrap(); 
    println!("{}", &tree);

    let points = tree.get_points(-100..100, vars.iterations);
    println!("points: {:?}", points);
    
    draw_graph(&mut vars, &points).await;
}

async fn draw_graph(vars: &mut Vars, points: &Vec<(f32, f32)>) {
    let mut current_scale = 1.0;

    let mut zoom_level = 1.0;
    let dt = get_frame_time();

    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, screen_size().1, screen_size().0, -screen_size().1));

    loop {

        if is_key_down(KeyCode::W) {zoom_level *= 1.0 + (dt * 1.0)}
        if is_key_down(KeyCode::S) {zoom_level *= 1.0 - (dt * 1.0)}
        zoom_level = zoom_level.clamp(0.5, 7.0);

        cam.zoom = vec2(zoom_level / screen_width() * 2.0, zoom_level / screen_height() * 2.0);
        set_camera(&cam);

        vars.update(current_scale);
        let mut points_it = points.into_iter().peekable();
        
        // Get world-space coordinates of the camera bounds
        vars.view = (cam.screen_to_world(vec2(0.0, 0.0)), cam.screen_to_world(vec2(screen_width(), screen_height())));
        vars.cam_mid = cam.screen_to_world(vec2(vars.mid.0, vars.mid.1));

        clear_background(BLACK);
        grid(&vars);
        axis(&vars);

        while let Some(current) = points_it.next() {
            if let Some(next) = points_it.peek() {
                let (x1, y1) = calc_cords(&vars, current);
                let (x2, y2) = calc_cords(&vars, next);
                draw_line(x1, y1, x2, y2, 1.2, BLUE);
            }
        } 

        set_default_camera();
        draw_text_ex("'W' and 'S'for zooming", 30., 30., TextParams {
            font_size: 20. as u16, 
            color: YELLOW,
            ..Default::default()
        });
        
        next_frame().await;
    }
}

fn axis(vars: &Vars) {
    let view = vars.view; 
    let real_mid = vars.cam_mid;
    // Draw the main axis lines
    draw_line(view.0.x, real_mid.y, view.1.x, real_mid.y, 1.0, WHITE);
    draw_line(real_mid.x, view.0.y, real_mid.x, view.1.y, 1.0, WHITE);

    // Compute how many steps fit within the visible world space
    let x_steps = ((view.1.x - real_mid.x) / vars.step).ceil() as i32; 
    let y_steps = ((view.1.y - real_mid.y) / vars.step).ceil() as i32; 

    // X-axis ticks (left & right from center)
    for i in -x_steps..=x_steps {
        if i == 0 {continue;}
        let x = real_mid.x + i as f32 * vars.step;
        draw_line(x, real_mid.y - 7.0, x, real_mid.y + 7.0, 1.0, WHITE);
        draw_text(&i.to_string(), x - vars.font_size / 2.0, real_mid.y - vars.font_size - 2.0, 18.0 * vars.scale, WHITE);
    }

    // Y-axis ticks (up & down from center)
    for i in -y_steps..=y_steps {
        if i == 0 {continue;}
        let y = real_mid.y + i as f32 * vars.step;
        draw_line(real_mid.x - 7.0, y, real_mid.x + 7.0, y, 1.0, WHITE);
        draw_text(&(-i).to_string(), real_mid.x + vars.font_size + 2.0, y + vars.font_size / 2.0, 18.0 * vars.scale, WHITE);
    }
}


fn grid(vars: &Vars) {
    let grey = Color::new(163.0, 163.0, 163.0, 0.05);
    let x_steps = ((vars.view.1.x - vars.cam_mid.x) / vars.step).ceil() as i32; // Steps right of mid
    let y_steps = ((vars.view.1.y - vars.cam_mid.y) / vars.step).ceil() as i32; // Steps above mid


    for i in -x_steps..=x_steps {
        let x = vars.cam_mid.x + i as f32 * vars.step;
        draw_line(x, vars.view.0.y, x, vars.view.1.y, 1.0, grey);
    }

    for i in -y_steps..=y_steps {
        let y = vars.cam_mid.y + i as f32 * vars.step;
        draw_line(vars.view.0.x, y, vars.view.1.x, y, 1.0, grey);
    }

}

fn calc_cords(vars: &Vars, point: &(f32, f32)) -> (f32, f32) {
    let x = vars.cam_mid.x + (point.0 * vars.step * vars.scale);
    let y = vars.cam_mid.y - (point.1 * vars.step * vars.scale);
    (x, y)
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Graph".to_owned(),
        window_width: 1200,
        window_height: 900,
        sample_count: 16,
        ..Default::default()
    }

}

fn combine_numbers_and_chars(input: Vec<char>) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_number = String::new();
    //combine all numbers next to each other to a single one
    for ch in input {
        if ch.is_ascii_digit() {
            current_number.push(ch);
        } else {
            if !current_number.is_empty() {
                result.push(current_number.clone());
                current_number.clear();
            }
            result.push(ch.to_string());
        }
    }
    if !current_number.is_empty() {
        result.push(current_number);
    }
    result
}
