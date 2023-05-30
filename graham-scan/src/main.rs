use clap::{Args, Parser, Subcommand, ValueEnum};
use rand::distributions::{Distribution, Uniform};
use rand_distr::Normal;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

mod points;
use points::Point;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
#[command(propagate_version = true)]
struct CLI {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run(RunArgs),
    Sample(SampleArgs),
}

#[derive(Args)]
struct RunArgs {
    #[arg(default_value = "points.txt")]
    filename: String,
}

#[derive(Args)]
struct SampleArgs {
    #[arg(default_value = "points.txt")]
    filename: String,
    #[arg(value_enum, default_value_t=Sampler::Random)]
    sampler: Sampler,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Sampler {
    Random,
    Normal,
}

impl fmt::Display for Sampler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Sampler::Random => write!(f, "Random"),
            Sampler::Normal => write!(f, "Normal"),
        }
    }
}

const DIMS: (i32, i32) = (256, 256);

fn main() {
    // 1. Run graham scan on input file's point positions
    // 2. Sample point positions
    //   a. Specify point positions sample function
    let cli = CLI::parse();

    match &cli.command {
        Commands::Run(args) => {
            //run(*args.filename);
            run(&args.filename);
        }
        Commands::Sample(args) => {
            sample(&args.filename, &args.sampler);
        }
    }
}

fn run(filename: &str) {
    // runs graham scan upon points found in file
    println!("run - input from {}", filename);
    let Ok(samples) = load_point_vec(filename) else{
        panic!("Error loading from {}", filename);
    };
    graham_scan(samples);
}

fn sample(filename: &str, sampler: &Sampler) {
    // samples points from specified sampler and saves to file
    println!("sample - output to {}, sampler used {}", filename, sampler);
    let samples: Vec<Point> = match &sampler {
        Sampler::Random => generate_random_samples(200),
        Sampler::Normal => generate_normal_samples(200),
    };

    match save_point_vec(filename, samples) {
        Ok(()) => println!("Successfully wrote to {}", filename),
        Err(e) => println!("Error writing to {}: {:?}", filename, e),
    };
}

fn generate_normal_samples(n_points: u8) -> Vec<Point> {
    // generate a vector of n_points sampled from 2d gaussian
    println!("sampling points from Normal");
    let mut rng = rand::thread_rng();
    let normal_x = Normal::new((DIMS.0 / 2) as f32, (DIMS.0 / 10) as f32).unwrap();
    let normal_y = Normal::new((DIMS.1 / 2) as f32, (DIMS.1 / 10) as f32).unwrap();
    let mut samples: Vec<Point> = Vec::new();
    for _ in 0..n_points {
        let rx: i32 = normal_x.sample(&mut rng) as i32;
        let ry: i32 = normal_y.sample(&mut rng) as i32;
        samples.push(Point::new(rx, ry));
    }
    samples
}

fn generate_random_samples(n_points: u8) -> Vec<Point> {
    // generate a vector of n_points sampled from 2d gaussian
    println!("sampling points from random uniform");
    let mut rng = rand::thread_rng();
    let uniform_x = Uniform::from(0..DIMS.0);
    let uniform_y = Uniform::from(0..DIMS.0);
    let mut samples: Vec<Point> = Vec::new();
    for _ in 0..n_points {
        let rx: i32 = uniform_x.sample(&mut rng);
        let ry: i32 = uniform_y.sample(&mut rng);
        samples.push(Point::new(rx, ry));
    }
    samples
}

fn save_point_vec(filename: &str, point_vec: Vec<Point>) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    for point in &point_vec {
        let point_str = point.to_string();
        file.write_all(point_str.as_bytes())?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

// Todo here and throughout - better to take &str
fn load_point_vec(filename: &str) -> std::io::Result<Vec<Point>> {
    let mut points: Vec<Point> = Vec::new();
    let mut file = File::open(filename)?;
    let mut filestring = String::new();
    file.read_to_string(&mut filestring)?;
    let lines = filestring.lines();
    for point_str in lines {
        let point_string: String = String::from(point_str);
        points.push(point_string.parse().unwrap());
    }
    Ok(points)
}

fn dedup_by_angle_metric(base_point: Point, sorted_points: Vec<(Point, f32)>) -> Vec<Point> {
    // keep the furthest colinear points
    // discard the angle metric
    let mut deduped: Vec<Point> = vec![];
    deduped.push(sorted_points[0].0);
    let mut last_angle: f32 = sorted_points[0].1;
    let mut last_mag: f32 = (base_point - sorted_points[0].0).mag();
    for (p, a) in &sorted_points[1..] {
        let new_mag = (*p - base_point).mag();
        if *a == last_angle {
            // same angle, only keep the furthest point
            if new_mag > last_mag {
                deduped.pop(); // discard closer point that is colinear with base point
                deduped.push(*p); //
            }
        } else {
            // different angle, keep for now
            deduped.push(*p);
        }
        last_mag = new_mag;
        last_angle = *a;
    }
    deduped
}

pub fn graham_scan(points: Vec<Point>) -> Vec<Point> {
    // we have a vector of points as argument
    // we have a stack of convex hull points
    // 1. we find the base point and remove it, adding to convex hull
    // 2. we make a new vector (point,angle) and sort
    // 2. a. remove the nearer colinear point
    // 3. we find the first point and remove it, adding to convex hull
    // 4. we loop over this vector to push to convex hull and make checks.

    if points.len() < 3 {
        panic!("Minimum of 3 points required for convex hull");
    }

    let mut convex_hull: Vec<Point> = vec![];

    // 1. find base
    let Some(base_point) = points.iter().min() else{
        panic!("No minimum point");
    };
    println!("base point {}", base_point);
    convex_hull.push(*base_point);

    // 2. angles and sort
    let cand_points: Vec<(Point, f32)> = points
        .iter()
        .filter(|p| **p != *base_point)
        .map(|p| (*p, Point::cos(*p - *base_point, Point::new(1, 0))))
        .collect();

    let mut sorted_cand_points = cand_points;
    sorted_cand_points.sort_by(|(_p1, angle1), (_p2, angle2)| angle1.partial_cmp(angle2).unwrap());
    sorted_cand_points.reverse();

    // 2.a remove all but one if angle is the same- keep furthest point.
    // Can't concieve how to do this with iterator functions
    let candidates: Vec<Point> = dedup_by_angle_metric(*base_point, sorted_cand_points);

    let mut cand_iter = candidates.iter();
    //let mut candidates = sorted_cand_points.iter().map(|(p,_a)| p);

    // 3. we can push the first one to the convex hull because we know it will be in the convex hull
    let Some(first_cand) = cand_iter.next() else{
        panic!("Empty candidate iter");
    };
    convex_hull.push(*first_cand);
    // 4. do the scan
    let mut ch_len;
    for cand in cand_iter {
        convex_hull.push(*cand);
        ch_len = convex_hull.len();
        while ch_len >= 3 && is_right_or_no_turn(&convex_hull[ch_len - 3..ch_len]) {
            let Some(top) = convex_hull.pop() else{
                panic!("Empty convex hull");
            };
            let Some(_discard) = convex_hull.pop() else{
                panic!("Empty convex hull");
            };
            convex_hull.push(top);
            ch_len = convex_hull.len();
        }
    }

    for p in convex_hull.iter() {
        println!("{}", p);
    }
    println!(
        "Final Length of convex hull over {} points {}",
        points.len(),
        convex_hull.len()
    );

    convex_hull
}

fn is_right_or_no_turn(section: &[Point]) -> bool {
    // determine if path p1->p2->p3 constitutes a right turn
    // compute the z coordinate of the cross product of the two vectors p1p2 p1p3
    // (x2-x1)(y3-y1) - (y2-y1)(x3-x1)
    let p1: Point = section[0];
    let p2: Point = section[1];
    let p3: Point = section[2];
    let cross_z: i32 = Point::cross_z(p1, p2, p3);
    //let message;
    // if cross_z<0{
    //     message = "right turn";
    // }
    // else if cross_z >0 {
    //     message = "left turn";
    // }
    // else{
    //     message = "colinear";
    // }
    // println!("{}->{}->{} is {}", p1,p2,p3,message);
    cross_z <= 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_points(positions: Vec<(i32, i32)>) -> Vec<Point> {
        let mut points: Vec<Point> = vec![];
        for (x, y) in positions {
            points.push(Point::new(x, y));
        }
        points
    }

    #[test]
    #[should_panic]
    fn panic_1_point() {
        let points = vec![(10, 10)];
        graham_scan(make_points(points));
    }

    #[test]
    #[should_panic]
    fn panic_2_point() {
        let points = vec![(10, 10), (20, 10)];
        graham_scan(make_points(points));
    }

    #[test]
    fn basic_square() {
        let square_points = vec![(10, 10), (50, 10), (50, 50), (10, 50)];
        let test_points: Vec<Point> = make_points(square_points.clone());
        let expected_points: Vec<Point> = make_points(square_points);
        let res: Vec<Point> = graham_scan(test_points);
        assert_eq!(expected_points, res);
    }

    #[test]
    fn colinear_square() {
        // Convex hull should be minimum convex set that contains
        let square_points = vec![(10, 10), (50, 10), (50, 50), (10, 50)];
        let points = vec![
            (10, 10),
            (50, 10),
            (50, 50),
            (10, 50),
            (30, 10),
            (50, 30),
            (30, 50),
            (10, 30),
        ];
        let test_points: Vec<Point> = make_points(points);
        let expected_points: Vec<Point> = make_points(square_points);
        let res: Vec<Point> = graham_scan(test_points);
        assert_eq!(expected_points, res);
    }

    #[test]
    fn dedupe_simple() {
        let duplicate_angles: Vec<(Point, f32)> =
            vec![(Point::new(10, 10), 0.5), (Point::new(20, 20), 0.5)];
        let res: Vec<Point> = dedup_by_angle_metric(Point::new(0, 0), duplicate_angles);
        assert_eq!(res, vec![Point::new(20, 20)]);
    }

    #[test]
    fn dedupe_more() {
        let duplicate_angles: Vec<(Point, f32)> = vec![
            (Point::new(10, 10), 0.5),
            (Point::new(20, 20), 0.5),
            (Point::new(16, 20), 0.4),
            (Point::new(15, 20), 0.3),
            (Point::new(30, 40), 0.3),
        ];
        let res: Vec<Point> = dedup_by_angle_metric(Point::new(0, 0), duplicate_angles);
        assert_eq!(
            res,
            vec![Point::new(20, 20), Point::new(16, 20), Point::new(30, 40)]
        );
    }
}
