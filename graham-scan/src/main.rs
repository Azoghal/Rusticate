use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt;
//use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use rand_distr::{Normal};
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::cmp::Ordering;
use core::ops;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
#[command(propagate_version = true)]
struct CLI{
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run(RunArgs),
    Sample(SampleArgs),
}

#[derive(Args)]
struct RunArgs{
    filename:String,
}

#[derive(Args)]
struct SampleArgs{
    filename:String,
    #[arg(value_enum, default_value_t=Sampler::Random)]
    sampler:Sampler,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Sampler{
    Random,
    Normal,
}

impl fmt::Display for Sampler{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self{
            Sampler::Random => write!(f,"Random"),
            Sampler::Normal => write!(f,"Normal"),
        }
    }
}


const DIMS:(i32,i32) = (256,256);

fn main() {
    // TODO: parse arguments that determine action
    // 1. Run graham scan on input file's point positions
    // 2. Generate point positions
    //   a. Specify point positions sample function
    let cli = CLI::parse();

    match &cli.command{
        Commands::Run(args) => {
            //run(*args.filename);
            run(&args.filename);
        },
        Commands::Sample(args) => {
            sample(&args.filename, &args.sampler);
        }
    }
}

// copy and clone are reasonable as in rusttype::Point
#[derive(Copy, Clone, Debug, Eq)] 
pub struct Point{
    x:i32,
    y:i32,
}

impl Point{
    fn mag(&self)->f32{
        return ((self.x*self.x + self.y*self.y)as f32).sqrt();
    }

    fn dot(&self, rhs:Point) -> i32{
        self.x*rhs.x + self.y*rhs.y
    }
}

impl Ord for Point{
    fn cmp(&self, other: &Self) -> Ordering{
        self.y.cmp(&other.y).then(self.x.cmp(&other.x))
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl ops::Sub<Point> for Point{
    type Output = Point;
    fn sub(self, rhs: Point) -> Point{
        Point{x:self.x-rhs.x, y:self.y-rhs.y}
    }
}

impl ops::Add<Point> for Point{
    type Output = Point;
    fn add(self, rhs: Point) -> Point{
        Point{x:self.x+rhs.x, y:self.y+rhs.y}
    }
}

impl fmt::Display for Point{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f,"({},{})", self.x, self.y)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsePointError;

impl FromStr for Point{
    type Err = ParsePointError;
    fn from_str(s: &str) -> Result<Self,Self::Err>{
        let (x, y) = s
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.split_once(','))
            .ok_or(ParsePointError)?;
        let x_fromstr = x.parse::<i32>().map_err(|_| ParsePointError)?;
        let y_fromstr = y.parse::<i32>().map_err(|_| ParsePointError)?;
        Ok(Point{x:x_fromstr, y:y_fromstr})
    }
}


fn run(filename:&String){
    // runs graham scan upon points found in file
    println!("run - input from {}",filename);
    let Ok(samples) = load_point_vec(filename) else{
        panic!("Error loading from {}", filename);
    };
    graham_scan(samples);
}


fn sample(filename:&String, sampler: &Sampler){
    // samples points from specified sampler and saves to file
    println!("sample - output to {}, sampler used {}", filename, sampler);
    let samples:Vec<Point> = match &sampler{
        Sampler::Random => {
            generate_random_samples(200)
        },
        Sampler::Normal => {
            generate_normal_samples(200)
        },
    };

    match save_point_vec(filename, samples){
        Ok(())=>println!("Successfully wrote to {}", filename),
        Err(e)=>println!("Error writing to {}: {:?}", filename, e),
    };
}

fn generate_normal_samples(n_points:u8)-> Vec<Point>{
    // generate a vector of n_points sampled from 2d gaussian
    println!("sampling points from Normal");
    let mut rng = rand::thread_rng();
    let normal_x = Normal::new((DIMS.0/2) as f32,(DIMS.0/10) as f32).unwrap();
    let normal_y = Normal::new((DIMS.1/2) as f32,(DIMS.1/10) as f32).unwrap();
    let mut samples:Vec<Point> = Vec::new();
    for _ in 0..n_points{
        // Todo replace with normal samples
        let rx:i32 = normal_x.sample(&mut rng) as i32;
        let ry:i32 = normal_y.sample(&mut rng) as i32;
        samples.push(Point{x:rx,y:ry});
    }
    samples
}

fn generate_random_samples(n_points:u8)-> Vec<Point>{
    // generate a vector of n_points sampled from 2d gaussian
    println!("sampling points from random uniform");
    let mut rng = rand::thread_rng();
    let uniform_x = Uniform::from(0..DIMS.0);
    let uniform_y = Uniform::from(0..DIMS.0);
    let mut samples:Vec<Point> = Vec::new();
    for _ in 0..n_points{
        // Todo replace with random samples
        let rx:i32 = uniform_x.sample(&mut rng) as i32;
        let ry:i32 = uniform_y.sample(&mut rng) as i32;
        samples.push(Point{x:rx,y:ry});
    }
    samples
}

fn save_point_vec(filename:&String, point_vec:Vec<Point>) -> std::io::Result<()>{
    let mut file = File::create(filename)?;
    for point in &point_vec{
        let point_str = point.to_string();
        file.write(point_str.as_bytes())?;
        file.write(b"\n")?;
    }
    Ok(())
}

// Todo here and throughout - better to take &str
fn load_point_vec(filename:&String) -> std::io::Result<Vec<Point>>{
    let mut points:Vec<Point> = Vec::new();
    let mut file = File::open(filename)?;
    let mut filestring = String::new();
    file.read_to_string(&mut filestring)?;
    let lines = filestring.lines();
    for point_str in lines{
        let point_string: String = String::from(point_str);
        points.push(point_string.parse().unwrap());
    }
    Ok(points)
}

// todo: make part of Points struct/class
// fn points_mag(p:&Point)->f32{
//     let mag_sq = (p.x*p.x + p.y*p.y) as f32;
//     let mag:f32 = mag_sq.sqrt();
//     return mag;
// }

// // TODO remove by adding operators for point
// fn points_dif_mag(p1:&Point, p2:&Point)->f32{
//     let p = Point{x:p1.x-p2.x, y:p1.y-p2.y};
//     return points_mag(&p);
// }

// fn points_dot(p1:&Point, p2:&Point)->i32{
//     return p1.x*p2.x + p1.y*p2.y;
// }

fn points_cos(base:&Point, p1:&Point)->f32{
    // TODO: Point operators
    let dif: Point = *p1-*base;
    let x_axis = Point{x:1,y:0};
    let mag_prod = dif.mag();
    let dot = dif.dot(x_axis) as f32;
    let cos = dot/mag_prod;
    return cos;
}

// alternative angle sorting metric
// fn _points_slope(p1: &Point, p2:&Point) -> f32{
//     //TODO refactor with proper operators
//     let dif = *p2-*p1;
//     return (dif.y as f32)/(dif.x as f32);
// }

fn dedup_by_angle_metric(base_point:Point, sorted_points: Vec<(Point, f32)>) -> Vec<Point>{
    // keep the furthest colinear points
    // discard the angle metric
    let mut deduped:Vec<Point>  = vec![];
    deduped.push(sorted_points[0].0);
    let mut last_angle: f32 = sorted_points[0].1;
    let mut last_mag: f32 = (base_point-sorted_points[0].0).mag();
    for (p, a) in &sorted_points[1..]{
        let new_mag = (*p-base_point).mag();
        if *a == last_angle{
            // same angle, only keep the furthest point
            if new_mag > last_mag{
                deduped.pop(); // discard closer point that is colinear with base point
                deduped.push(*p); // 
            }
        }else{
            // different angle, keep for now
            deduped.push(*p);
        }
        last_mag = new_mag;
        last_angle = *a;
    }
    return deduped;
}

pub fn graham_scan(points: Vec<Point>) -> Vec<Point> {
    // we have a vector of points as argument
    // we have a stack of convex hull points
    // 1. we find the base point and remove it, adding to convex hull
    // 2. we make a new vector (point,angle) and sort
    // 2. a. remove the nearer colinear point
    // 3. we find the first point and remove it, adding to convex hull
    // 4. we loop over this vector to push to convex hull and make checks.

    if points.len() < 3{
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
    let cand_points: Vec<(Point,f32)> = points
        .iter()
        .filter(|p| **p!=*base_point)
        .map(|p| (*p, points_cos(p, base_point)))
        .collect();

    let mut sorted_cand_points = cand_points.clone();
    sorted_cand_points.sort_by(|(_p1,angle1),(_p2,angle2)| angle1.partial_cmp(&angle2).unwrap());

    // 2.a remove all but one if angle is the same- keep furthest point.
    // Can't concieve how to do this with iterator functions
    let candidates:Vec<Point> = dedup_by_angle_metric(*base_point, sorted_cand_points);

    for c in candidates.iter(){
        println!("{}", c);
    }

    let mut cand_iter = candidates.iter();
    //let mut candidates = sorted_cand_points.iter().map(|(p,_a)| p);

    // 3. we can push the first one to the convex hull because we know it will be in the convex hull
    let Some(first_cand) = cand_iter.next() else{
        panic!("Empty candidate iter");
    };
    convex_hull.push(*first_cand);
    // 4. do the scan
    let mut ch_len;
    for cand in cand_iter{
        convex_hull.push(*cand);
        ch_len = convex_hull.len();
        while ch_len >= 3 && is_right_or_no_turn(&convex_hull[ch_len-3..ch_len]){
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

    for p in convex_hull.iter(){
        println!("{}",p);
    }
    println!("Final Length of convex hull over {} points {}", points.len(), convex_hull.len());

    return convex_hull;
}

fn is_right_or_no_turn(section: &[Point]) -> bool{
    // determine if path p1->p2->p3 constitutes a right turn
    // compute the z coordinate of the cross product of the two vectors p1p2 p1p3
    // (x2-x1)(y3-y1) - (y2-y1)(x3-x1)
    let p1: Point = section[0];
    let p2: Point = section[1];
    let p3: Point = section[2];
    let cross_z:i32 = (p2.x-p1.x)*(p3.y-p1.y) - (p2.y-p1.y)*(p3.x-p1.x); 
    let message;
    if cross_z<0{
        message = "right turn";
    }
    else if cross_z >0 {
        message = "left turn";
    }
    else{
        message = "colinear";
    }
    println!("{}->{}->{} is {}", p1,p2,p3,message);
    return cross_z<=0;
}


#[cfg(test)]
mod tests{
    use super::*;

    fn make_points(positions: Vec<(i32,i32)>) -> Vec<Point>{
        let mut points: Vec<Point> = vec![];
        for (x,y) in positions{
            points.push(Point{x,y});
        }
        return points;
    }

    #[test]
    #[should_panic]
    fn panic_1_point(){
        let points = vec![(10,10)];
        graham_scan(make_points(points));
    }

    #[test]
    #[should_panic]
    fn panic_2_point(){
        let points = vec![(10,10), (20,10)];
        graham_scan(make_points(points));
    }

    #[test]
    fn basic_square(){
        let square_points = vec![(10,10),(50,10),(50,50),(10,50)];
        let test_points: Vec<Point> = make_points(square_points.clone());
        let expected_points: Vec<Point> = make_points(square_points.clone());
        let res: Vec<Point> = graham_scan(test_points);
        assert_eq!(expected_points, res);
    }

    #[test]
    fn colinear_square(){
        // Convex hull should be minimum convex set that contains
        let square_points = vec![(10,10),(50,10),(50,50),(10,50)];
        let points = vec![(10,10),(50,10),(50,50),(10,50), (30,10),(50,30),(30,50),(10,30)];
        let test_points: Vec<Point> = make_points(points);
        let expected_points: Vec<Point> = make_points(square_points);
        let res: Vec<Point> = graham_scan(test_points);
        assert_eq!(expected_points, res);
    }

    #[test]
    fn dedupe_simple(){
        let duplicate_angles: Vec<(Point,f32)> = vec![(Point{x:10,y:10}, 0.5), (Point{x:20,y:20}, 0.5)];
        let res = dedup_by_angle_metric(Point{x:0,y:0}, duplicate_angles);
        assert_eq!(res, vec![Point{x:20,y:20}]);
    }

    #[test]
    fn dedupe_more(){
        let duplicate_angles: Vec<(Point,f32)> = vec![
            (Point{x:10,y:10}, 0.5), (Point{x:20,y:20}, 0.5), (Point{x:16,y:20}, 0.4),(Point{x:15,y:20}, 0.3), (Point{x:30,y:40}, 0.3)
        ];
        let res = dedup_by_angle_metric(Point{x:0,y:0}, duplicate_angles);
        assert_eq!(res, vec![Point{x:20,y:20},Point{x:16,y:20},Point{x:30,y:40}]);
    }

}