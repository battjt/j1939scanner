use std::option::*;
use std::sync::*;

struct MQItem<T> {
    data: T,
    next: Arc<RwLock<Option<MQItem<T>>>>,
}

pub struct MultiQueue<T> {
    head: Arc<RwLock<Option<MQItem<T>>>>,
}
impl<T> MQItem<T> {
    fn new(data: T) -> MQItem<T> {
        MQItem {
            data,
            next: Arc::new(RwLock::new(None)),
        }
    }
    fn next(&self) -> Arc<RwLock<Option<MQItem<T>>>> {
        self.next.clone()
    }
}
impl<T> MQItem<T> {
    fn push(&self, item: T) {
        let mut n = self.next.write().unwrap();
        if n.is_some() {
            n.as_ref().unwrap().push(item)
        } else {
            *n = Some(MQItem::new(item));
        }
    }
}
impl<T> MultiQueue<T>
where
    T: Copy,
{
    pub fn new() -> MultiQueue<T> {
        MultiQueue {
            head: Arc::new(RwLock::new(None)),
        }
    }
    pub fn pull(&mut self) -> Option<T> {
        let o = self
            .head
            .read()
            .unwrap()
            .as_ref()
            .map(|i| (i.next.clone(), i.data));
        o.map(|i| {
            self.head = i.0;
            i.1
        })
    }
    pub fn push(&mut self, item: T) {
        let mut opt = self.head.write().unwrap();
        if opt.is_some() {
            opt.as_ref().unwrap().push(item)
        } else {
            *opt = Some(MQItem::new(item));
        }
    }
    pub fn clone(&self) -> Self {
        MultiQueue {
            head: self.head.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut q: MultiQueue<&str> = MultiQueue::new();
        q.push("one");
        // let mut q = q.clone();
        q.push("two");
        q.push("three");
        assert_eq!("two", q.pull().unwrap());
        assert_eq!(std::option::Option::None, q.pull());
    }
}
