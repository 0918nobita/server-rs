use super::Job;

pub enum Message {
    NewJob(Job),
    Terminate,
}
