use regex::RegexBuilder;
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
            let job = Job::from_str_parts(parts);
            return Some(job);
        })
        .collect();
    jobs
}
