use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use ratatui::layout::Size;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{FilterType, Resize, ResizeEncodeRender};

pub fn resize() -> Resize {
    Resize::Scale(Some(FilterType::Triangle))
}

struct Job {
    id: String,
    path: PathBuf,
    size: Size,
}

struct Done {
    id: String,
    size: Size,
    protocol: Option<StatefulProtocol>,
}

// Encoding a preview stalls key handling, so a worker does it; results are cached per theme and size.
pub struct Previews {
    jobs: Option<Sender<Job>>,
    done: Option<Receiver<Done>>,
    ready: HashMap<String, (Size, Option<StatefulProtocol>)>,
    pending: HashSet<(String, Size)>,
}

impl Previews {
    pub fn start(picker: Picker) -> Self {
        let (jobs, job_rx) = channel::<Job>();
        let (done_tx, done) = channel::<Done>();
        std::thread::spawn(move || {
            for job in job_rx {
                let protocol = image::ImageReader::open(&job.path)
                    .ok()
                    .and_then(|r| r.decode().ok())
                    .map(|img| {
                        let mut p = picker.new_resize_protocol(img);
                        p.resize_encode(&resize(), job.size);
                        p
                    });
                if done_tx
                    .send(Done {
                        id: job.id,
                        size: job.size,
                        protocol,
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            jobs: Some(jobs),
            done: Some(done),
            ready: HashMap::new(),
            pending: HashSet::new(),
        }
    }

    #[cfg(test)]
    pub fn disabled() -> Self {
        Self {
            jobs: None,
            done: None,
            ready: HashMap::new(),
            pending: HashSet::new(),
        }
    }

    pub fn poll(&mut self) -> bool {
        let Some(done) = &self.done else { return false };
        let mut any = false;
        while let Ok(d) = done.try_recv() {
            self.pending.remove(&(d.id.clone(), d.size));
            self.ready.insert(d.id, (d.size, d.protocol));
            any = true;
        }
        any
    }

    pub fn clear(&mut self) {
        self.ready.clear();
        self.pending.clear();
    }

    pub fn get(
        &mut self,
        id: &str,
        path: Option<PathBuf>,
        size: Size,
        may_request: bool,
    ) -> Lookup<'_> {
        let fresh = self.ready.get(id).is_some_and(|(s, _)| *s == size);
        if fresh {
            return match self.ready.get_mut(id).and_then(|(_, p)| p.as_mut()) {
                Some(p) => Lookup::Ready(p),
                None => Lookup::Missing,
            };
        }
        let Some(path) = path else {
            return Lookup::Missing;
        };
        let key = (id.to_string(), size);
        if may_request
            && !self.pending.contains(&key)
            && let Some(jobs) = &self.jobs
        {
            let _ = jobs.send(Job {
                id: id.to_string(),
                path,
                size,
            });
            self.pending.insert(key);
        }
        Lookup::Loading
    }
}

pub enum Lookup<'a> {
    Ready(&'a mut StatefulProtocol),
    Loading,
    Missing,
}
