use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt;
//use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use rand_distr::{Normal};
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::cmp::Ordering;


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
struct Point{
    x:i32,
    y:i32,
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

impl fmt::Display for Point{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f,"({},{})", self.x, self.y)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParsePointError;

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
fn points_mag(p:&Point)->f32{
    let mag_sq = (p.x*p.x + p.y*p.y) as f32;
    let mag:f32 = mag_sq.sqrt();
    return mag;
}

fn points_dot(p1:&Point, p2:&Point)->i32{
    return p1.x*p2.x + p1.y*p2.y;
}

fn points_cos(p1:&Point, p2:&Point)->f32{
    let mag_prod = points_mag(&p1) * points_mag(&p2);
    let dot = points_dot(&p1, &p2) as f32;
    let cos = dot/mag_prod;
    return cos;
}

fn graham_scan(points:Vec<Point>){
    // take all the points
    // return an ordered vector of convex hull points,
    // counterclockwise from lowest point.

    // TODO: tidy this whole thing up with mega tuples in one vec?
    // Might not be any tidier.

    let i_points: Vec<(usize,&Point)> = points.iter().enumerate().collect();

    let min_y_index : Option<&usize> = i_points
        .iter()
        .min_by(|(_, a), (_, b)| a.cmp(b))
        .map(|(index, _)| index);
    

    let Some(idx) = min_y_index else{
        panic!("Error finding min y");
    };
    let (base_id, base_point) = i_points[*idx];
    println!("base point {} {}", base_id, base_point);
    
    let mut angles: Vec<(usize,f32)>= i_points.iter().map(|(i,p)| (*i ,points_cos(p,&base_point))).collect();
    // for(i,ang) in &angles{
    //     println!("{} cos: {}",i, ang);
    // }

    // println!("Now sorting");
    angles.sort_by(|(_i,a),(_j,b)| a.partial_cmp(&b).unwrap());
    angles.reverse();
    // for(i,ang) in &angles{
    //     println!("{} at {} cos: {}",i, i_points[*i].1, ang);
    // }

    let order: Vec<usize> = angles.iter().map(|(i,_)| *i).collect();
    let ordered_points: Vec<&Point> = order.iter().map(|i| &points[*i]).collect();

    let mut convex_hull: Vec<&Point> = vec![]; // where we're going to store our convex hull
    // we add the lowest point, and the first one we hit as we raise angle from the x axis
    // as these are guarunteed to be in the hull.
    convex_hull.push(ordered_points[0]);
    convex_hull.push(ordered_points[1]);
    // the last element of the convex hull should be the last element of the ordered points
    // by the same argument as that ordered_points[1] will be in the convex hull.
    
    println!("Starting the scan");

    //do the scan.
    let mut ch_len;
    for i in 2..ordered_points.len(){
        ch_len = convex_hull.len();
        convex_hull.push(ordered_points[i]);
        while ch_len >= 3 && is_right_turn(&convex_hull[ch_len-3..ch_len]){
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
}

fn is_right_turn(section: &[&Point]) -> bool{
    // determine if path p1->p2->p3 constitutes a left or right turn
    // compute the z coordinate of the cross product of the two vectors p1p2 p1p3
    // (x2-x1)(y3-y1) - (y2-y1)(x3-x1)
    let p1: &Point = section[0];
    let p2: &Point = section[1];
    let p3: &Point = section[2];
    let cross_z:i32 = (p2.x-p1.x)*(p3.y-p1.y) - (p2.y-p1.y)*(p3.x-p1.x);   
    return cross_z<0;
}