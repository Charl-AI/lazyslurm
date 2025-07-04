use regex::{RegexBuilder,Regex};
use std::collections::HashMap;
use std::{io::BufRead, process::Command};

// This macro is a bit crazy.
// the reason we have it is because the names of the fields were
// being repeated in lots of places: the command given to squeue,
// constructing the struct, and using the struct.
// By converting the field names and values to vectors of strings,
// we can shorten the code. NB: this means that the name of each
// field should also be the command given to squeue to retrieve it.
macro_rules! make_field_names_available {
    (struct $name:ident {
        $($field_name:ident: $field_type:ty,)*
    }) => {
        #[allow(non_snake_case)]
        pub struct $name {
            $(pub $field_name: $field_type,)*
        }

        impl $name {
            pub fn field_names() -> Vec<&'static str> {
                vec![$(stringify!($field_name)),*]
            }
            pub fn field_values(&self) -> Vec<String>{
                vec![
                    $(self.$field_name.clone().to_string()),*
                ]
            }
            fn from_str_parts(parts: Vec<&str>) -> Self {
                let mut iter = parts.into_iter();
                Self {
                    $($field_name: iter.next().unwrap().to_string()),*
                }
            }
        }
    }
}

make_field_names_available!(
    struct Job {
        StateCompact: String,
        State: String,
        Reason: String,
        Name: String,
        UserName: String,
        JobID: String,
        ArrayJobID: String,
        ArrayTaskID: String,
        Partition: String,
        NodeList: String,
        ReqNodes: String,
        SubmitTime: String,
        StartTime: String,
        TimeLimit: String,
        TimeUsed: String,
        TRES: String,
        NumTasks: String,
        Priority: String,
        WorkDir: String,
        Command: String,
        STDOUT: String,
        STDERR: String,
    }
);

pub fn get_jobs(filter_re: &String) -> Vec<Job> {
    let output_separator = "###";
    let fields = Job::field_names().to_owned();
    let output_format: Vec<String> = fields
        .iter()
        .map(|s| s.to_string().to_owned() + ":" + output_separator)
        .collect();
    let format_str: String = output_format.join(",");

    let re = RegexBuilder::new(filter_re)
        .case_insensitive(true)
        .build()
        // if invalid regex, just use ""
        .unwrap_or(RegexBuilder::new("").build().unwrap());

    let jobs: Vec<Job> = Command::new("squeue")
        .arg("--array")
        .arg("--noheader")
        .arg("--Format")
        .arg(format_str)
        .output()
        .expect("failed to execute squeue")
        .stdout
        .lines()
        .map(|l| l.unwrap().trim().to_string())
        .filter_map(|l| {
            if !re.is_match(l.as_str()) {
                return None;
            }
            let parts: Vec<_> = l.split(output_separator).collect();
            if parts.len() != fields.len() + 1 {
                return None;
            }
            let mut job = Job::from_str_parts(parts);
            parse_paths(&mut job);
            return Some(job);
        })
        .collect();
    jobs
}

// STDOUT as retrieved by squeue looks like this: slurm.%N.%j.log,
// we need to interpolate the terms into the actual path.
fn parse_paths(job: &mut Job) {
    job.STDOUT = interpolate_path(&job.STDOUT, job);
    job.STDERR = interpolate_path(&job.STDERR, job);
}

fn interpolate_path(pattern: &str, job: &Job) -> String {
    let mut result = String::new();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(next_c) = chars.peek() {
                match next_c {
                    '\\' => {
                        result.push(c);
                        chars.next(); // Consume the next character
                    }
                    '%' => {
                        result.push('%');
                        chars.next(); // Consume the next character
                    }
                    _ => {
                        let mut pad_specifier = String::new();
                        while let Some(d) = chars.peek() {
                            if d.is_digit(10) {
                                pad_specifier.push(*d);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        let specifier = chars.next().unwrap();
                        if let Some(mut replacement) = replace_char(specifier, job) {
                            if !pad_specifier.is_empty() && specifier.is_numeric() {
                                let width: usize = pad_specifier.parse().unwrap_or(0);
                                replacement = format!("{:0width$}", replacement, width = width);
                            }
                            result.push_str(&replacement);
                        } else {
                            result.push(c);
                            result.push(specifier);
                        }
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

// see `man squeue` for all valid interpolations
fn replace_char(symbol: char, job: &Job) -> Option<String> {
    match symbol {
        // NB this does not cover some valid cases (A,J,n,s,t)
        // because I'm not too sure how to do them atm.
        // this should be enough for most cases.
        'a' => Some(job.ArrayJobID.to_owned()),
        'N' => Some(job.NodeList.to_owned()),
        'u' => Some(job.UserName.to_owned()),
        'x' => Some(job.Name.to_owned()),
        'j' => Some(job.JobID.to_owned()),
        _ => None,
    }
}

#[derive(Clone, Debug, Default)]
pub struct PartitionInfo {
    pub name: String,
    pub gpus_alloc: u32,
    pub gpus_total: u32,
}

#[derive(Clone, Debug, Default)]
pub struct UserStats {
    pub name: String,
    pub running_jobs: u32,
    pub pending_jobs: u32,
    pub gpus_used: u32,
}

#[derive(Clone, Debug, Default)]
pub struct ClusterOverview {
    pub jobs_running: u32,
    pub jobs_pending: u32,
    pub jobs_completing: u32,
    pub partitions: Vec<PartitionInfo>,
    pub user_stats: Vec<UserStats>,
}

pub fn get_cluster_overview(jobs: &Vec<Job>) -> ClusterOverview {
    let mut overview = ClusterOverview::default();
    let mut partitions_map: HashMap<String, PartitionInfo> = HashMap::new();
    let mut user_stats_map: HashMap<String, UserStats> = HashMap::new();

    let sinfo_output = Command::new("sinfo")
        .arg("-N")
        .arg("-o")
        .arg("%P %G")
        .output()
        .expect("failed to execute sinfo");

    let sinfo_str = String::from_utf8_lossy(&sinfo_output.stdout);
    let gpu_re = Regex::new(r"gpu(:\w+)?:(\d+)").unwrap();

    for line in sinfo_str.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 { continue; }

        let partition_name = parts[0].to_string();
        let gres_str = parts[1];

        let partition_info = partitions_map
            .entry(partition_name.clone())
            .or_insert_with(|| PartitionInfo {
                name: partition_name,
                ..Default::default()
            });

        for cap in gpu_re.captures_iter(gres_str) {
            if let Some(gpu_count_match) = cap.get(2) {
                let gpus_on_node: u32 = gpu_count_match.as_str().parse().unwrap_or(0);
                partition_info.gpus_total += gpus_on_node;
            }
        }
    }

    let generic_gpu_re = Regex::new(r"gres/gpu=(\d+)").unwrap();
    let specific_gpu_re = Regex::new(r"gres/gpu:[^=]*=(\d+)").unwrap();

    for job in jobs {
        match job.State.as_str() {
            "RUNNING" => overview.jobs_running += 1,
            "PENDING" => overview.jobs_pending += 1,
            "COMPLETING" => overview.jobs_completing += 1,
            _ => (),
        }

        let user_stats = user_stats_map
            .entry(job.UserName.clone())
            .or_insert_with(|| UserStats {
                name: job.UserName.clone(),
                ..Default::default()
            });

        match job.State.as_str() {
            "RUNNING" => {
                user_stats.running_jobs += 1;

                let mut gpus_for_job = 0;
                if let Some(caps) = generic_gpu_re.captures(&job.TRES) {
                    if let Some(count_match) = caps.get(1) {
                        gpus_for_job = count_match.as_str().parse().unwrap_or(0);
                    }
                } else {
                    for caps in specific_gpu_re.captures_iter(&job.TRES) {
                        if let Some(count_match) = caps.get(1) {
                            gpus_for_job += count_match.as_str().parse::<u32>().unwrap_or(0);
                        }
                    }
                }

                user_stats.gpus_used += gpus_for_job;

                if let Some(partition_info) = partitions_map.get_mut(&job.Partition) {
                    partition_info.gpus_alloc += gpus_for_job;
                }
            }
            "PENDING" => {
                user_stats.pending_jobs += 1;
            }
            _ => (),
        }
    }

    let mut user_stats_vec: Vec<UserStats> = user_stats_map.into_values().collect();
    user_stats_vec.sort_by(|a, b| {
        b.gpus_used.cmp(&a.gpus_used)
            .then_with(|| a.name.cmp(&b.name))
    });
    overview.user_stats = user_stats_vec;

    overview.partitions = partitions_map.into_values().collect();
    overview.partitions.sort_by(|a, b| a.name.cmp(&b.name));
    overview
}
