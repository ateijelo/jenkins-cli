use std::{collections::HashMap, fmt::Display};

use anyhow::{bail, Result};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Url;
use urlencoding::decode;

lazy_static! {
    // static ref job_re: Regex = Regex::new(r"(job/[^/]+)+/(\d+)/?").unwrap();
    static ref SEGMENT: Regex = Regex::new(r"job/([^/]+)").unwrap();
    static ref CLASSIC_JOB: Regex = Regex::new(r"job/([^/]+)").unwrap();
    static ref BLUE_JOB: Regex = Regex::new(r"/blue/organizations/jenkins/([^/]+)").unwrap();
    static ref CLASSIC_BUILD: Regex = Regex::new(r"job/[^/]+/(\d+)").unwrap();
    static ref BLUE_BUILD: Regex = Regex::new(
        r"/blue/organizations/jenkins/[^/]+/detail/[^/]+/(\d+)/"
    ).unwrap();
}

type Params = HashMap<String, String>;

// A Jenkins job
#[derive(Debug, PartialEq, Eq)]
pub struct Job {
    path: Vec<String>,
    base_url: Url,
}

impl Job {
    pub fn parse(url: &str) -> Result<Job> {
        Job::new(&Url::parse(url)?)
    }

    pub fn new(url: &Url) -> Result<Job> {
        if url.path().starts_with("/job/") {
            let mut base_url = url.clone();
            base_url.set_path("");
            return Ok(Job {
                path: SEGMENT
                    .captures_iter(url.path())
                    .map(|c| c.get(1).unwrap().as_str().to_owned().replace("job/", ""))
                    .map(|x| decode(&x).expect("utf8").into_owned())
                    .collect(),
                base_url,
            });
        }

        if url.path().starts_with("/blue/") {
            if let Some(c) = BLUE_JOB.captures(url.path()) {
                let mut base_url = url.clone();
                base_url.set_path("");
                return Ok(Job {
                    path: c
                        .get(1)
                        .unwrap()
                        .as_str()
                        .split("%2F")
                        .map(|x| decode(x).expect("utf8").into_owned())
                        .collect(),
                    base_url,
                });
            }
        }
        bail!("Failed to parse job from url: {}", url);
    }

    pub fn build_path(&self, params: &Params) -> String {
        let mut path = format!("job/{}/build", self.path.join("/job/"));
        if !params.is_empty() {
            path.push_str("WithParameters");
        }
        path
    }
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.join(" » "))
    }
}

// A particular build of a Jenkins job
#[derive(Debug, PartialEq, Eq)]
pub struct JobBuild {
    job: Job,
    number: u32,
}

impl JobBuild {
    pub fn parse(url: &str) -> Result<JobBuild> {
        JobBuild::new(&Url::parse(url)?)
    }
    pub fn new(url: &Url) -> Result<JobBuild> {
        if let Some(cap) = CLASSIC_BUILD.captures(url.path()) {
            let job = Job::new(url)?;
            let number: u32 = cap.get(1).unwrap().as_str().parse()?;

            return Ok(JobBuild { job, number });
        } else if let Some(cap) = BLUE_BUILD.captures(url.path()) {
            let job = Job::new(url)?;
            let number: u32 = cap.get(1).unwrap().as_str().parse()?;

            return Ok(JobBuild { job, number });
        }
        bail!("Failed to parse `{}` as a Jenkins job url", url);
    }

    pub fn log_path(&self, start: u32) -> Result<Url> {
        let path = format!(
            "job/{}/{}/logText/progressiveText?start={start}",
            self.job.path.join("/job/"),
            self.number
        );
        Ok(self.job.base_url.join(&path)?)
    }

    pub fn params_path(&self) -> Result<Url> {
        let path = format!(
            "job/{}/{}/api/json?tree=actions[parameters[name,value]]",
            self.job.path.join("/job/"),
            self.number
        );
        Ok(self.job.base_url.join(&path)?)
    }
}

impl Display for JobBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} #{}", self.job, self.number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classic_job_display() -> Result<()> {
        let u = Url::parse("http://jenkins.invalid/job/")?;

        let job = Job::new(&u.join("x")?)?;
        assert_eq!(format!("{job}"), "x");

        let job = Job::new(&u.join("x/")?)?;
        assert_eq!(format!("{job}"), "x");

        let job = Job::new(&u.join("x/2")?)?;
        assert_eq!(format!("{job}"), "x");

        let job = Job::new(&u.join("a/job/b")?)?;
        assert_eq!(format!("{job}"), "a » b");

        let job = Job::new(&u.join("a/job/b/")?)?;
        assert_eq!(format!("{job}"), "a » b");

        let job = Job::new(&u.join("a/job/b/2")?)?;
        assert_eq!(format!("{job}"), "a » b");

        Ok(())
    }

    #[test]
    fn test_job_build_path() -> Result<()> {
        let u = Url::parse("http://jenkins.invalid/job/")?;

        let params: Params = [
            ("a".to_owned(), "1".to_owned()),
            ("b".to_owned(), "2".to_owned()),
        ]
        .iter()
        .cloned()
        .collect();

        let no_params = Params::new();

        let job = Job::new(&u.join("x")?)?;
        assert_eq!(job.build_path(&no_params), "job/x/build");

        let job = Job::new(&u.join("x")?)?;
        assert_eq!(job.build_path(&params), "job/x/buildWithParameters");

        let job = Job::new(&u.join("x/job/y")?)?;
        assert_eq!(job.build_path(&no_params), "job/x/job/y/build");

        let job = Job::new(&u.join("x/job/y")?)?;
        assert_eq!(job.build_path(&params), "job/x/job/y/buildWithParameters");

        Ok(())
    }

    #[test]
    fn test_blue_job_display() -> Result<()> {
        let u = Url::parse("http://jenkins.invalid/blue/organizations/jenkins/")?;

        let job = Job::new(&u.join("x")?)?;
        assert_eq!(format!("{job}"), "x");

        let job = Job::new(&u.join("x/activity")?)?;
        assert_eq!(format!("{job}"), "x");

        let job = Job::new(&u.join("x/branches")?)?;
        assert_eq!(format!("{job}"), "x");

        let job = Job::new(&u.join("a%2Fb")?)?;
        assert_eq!(format!("{job}"), "a » b");

        let job = Job::new(&u.join("folder%20a%2Ffolder%20b")?)?;
        assert_eq!(format!("{job}"), "folder a » folder b");

        let job = Job::new(&u.join("folder%20a%2Ffolder%20b/2")?)?;
        assert_eq!(format!("{job}"), "folder a » folder b");

        Ok(())
    }

    #[test]
    fn test_classic_build_display() -> Result<()> {
        let u = Url::parse("http://jenkins.invalid/job/")?;

        let b = JobBuild::new(&u.join("x/2")?)?;
        assert_eq!(format!("{b}"), "x #2");

        let b = JobBuild::new(&u.join("x/2/")?)?;
        assert_eq!(format!("{b}"), "x #2");

        let job = JobBuild::new(&u.join("a/job/b/2")?)?;
        assert_eq!(format!("{job}"), "a » b #2");

        let job = JobBuild::new(&u.join("a/job/b/2/")?)?;
        assert_eq!(format!("{job}"), "a » b #2");

        Ok(())
    }

    #[test]
    fn test_blue_build_display() -> Result<()> {
        let u = Url::parse("http://jenkins.invalid/blue/organizations/jenkins/")?;

        let b = JobBuild::new(&u.join("x/detail/x/2/changes")?)?;
        assert_eq!(format!("{b}"), "x #2");

        let b = JobBuild::new(&u.join("x/detail/x/2/pipeline")?)?;
        assert_eq!(format!("{b}"), "x #2");

        let b = JobBuild::new(&u.join("folder%20a%2Fjob%20b/detail/job%20b/2/changes")?)?;
        assert_eq!(format!("{b}"), "folder a » job b #2");

        let b = JobBuild::new(&u.join("folder%20a%2Fjob%20b/detail/job%20b/2/pipeline")?)?;
        assert_eq!(format!("{b}"), "folder a » job b #2");

        Ok(())
    }

    #[test]
    fn test_log_path() -> Result<()> {
        let u = Url::parse("http://jenkins.invalid/blue/organizations/jenkins/")?;
        let b = JobBuild::new(&u.join("x/detail/x/2/changes")?)?;
        assert_eq!(
            b.log_path(0)?,
            Url::parse("http://jenkins.invalid/job/x/2/logText/progressiveText?start=0")?
        );
        let b = JobBuild::new(&u.join("folder%20a%2Fjob%20b/detail/job%20b/2/changes")?)?;
        assert_eq!(
            b.log_path(0)?,
            Url::parse(
                "http://jenkins.invalid/job/folder a/job/job b/2/logText/progressiveText?start=0"
            )?
        );

        let u = Url::parse("http://jenkins.invalid/job/")?;
        let b = JobBuild::new(&u.join("x/2")?)?;
        assert_eq!(
            b.log_path(0)?,
            Url::parse("http://jenkins.invalid/job/x/2/logText/progressiveText?start=0")?
        );
        let b = JobBuild::new(&u.join("a/job/b/2")?)?;
        assert_eq!(
            b.log_path(0)?,
            Url::parse("http://jenkins.invalid/job/a/job/b/2/logText/progressiveText?start=0")?
        );

        Ok(())
    }
}
