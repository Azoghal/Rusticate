use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt;
//use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use rand_distr::{Normal};
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;

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

#[derive(Debug, PartialEq)]
struct Point{
    x:i32,
    y:i32,
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
    let i_points: Vec<(usize,&Point)> = points.iter().enumerate().collect();

    let min_y_index : Option<&usize> = i_points
        .iter()
        .map(|(i,p)| (i,p.y))
        .min_by(|(_, a), (_, b)| a.cmp(b))
        .map(|(index, _)| index);

    let Some(idx) = min_y_index else{
        panic!("Error finding min y");
    };
    
    let (_base_id, base_point) = i_points[*idx];
    
    let mut angles: Vec<(&usize,f32)>= i_points.iter().map(|(i,p)| (i,points_cos(p,&base_point))).collect();
    // for(i,ang) in &angles{
    //     println!("{} cos: {}",i, ang);
    // }

    // println!("Now sorting");
    angles.sort_by(|(_i,a),(_j,b)| a.partial_cmp(&b).unwrap());
    angles.reverse();
    // for(i,ang) in &angles{
    //     println!("{} cos: {}",i, ang);
    // }

    // now we have the order to consider points in, so we begin the scan.

}