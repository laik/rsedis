use std::fmt;
use std::error::Error;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::str::from_utf8;
use std::str::Utf8Error;
use std::num::ParseIntError;
use std::sync::mpsc::Sender;

use rand::random;
use skiplist::SkipMap;

use super::config::Config;
use super::util::Float;
use super::util::glob_match;
use super::util::mstime;

#[derive(PartialEq, Debug)]
pub enum Value {
    Nil,
    Integer(i64),
    Data(Vec<u8>),
    List(LinkedList<Vec<u8>>),
    Set(HashSet<Vec<u8>>),
    SortedSet(SkipMap<Float, Vec<u8>>, HashMap<Vec<u8>, Float>),
}

#[derive(Debug)]
pub enum OperationError {
    OverflowError,
    ValueError,
    WrongTypeError,
    OutOfBoundsError,
}

#[derive(PartialEq, Debug)]
pub enum PubsubEvent {
    Subscription(Vec<u8>, usize),
    Unsubscription(Vec<u8>, usize),
    PatternSubscription(Vec<u8>, usize),
    PatternUnsubscription(Vec<u8>, usize),
    Message(Vec<u8>, Option<Vec<u8>>, Vec<u8>),
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl Error for OperationError {
    fn description(&self) -> &str {
        return "oops";
    }
}

impl From<Utf8Error> for OperationError {
    fn from(_: Utf8Error) -> OperationError { OperationError::ValueError }
}

impl From<ParseIntError> for OperationError {
    fn from(_: ParseIntError) -> OperationError { OperationError::ValueError }
}

fn normalize_position(position: i64, _len: usize) -> Result<usize, usize> {
    let len = _len as i64;
    let mut pos = position;
    if pos < 0 {
        pos += len;
    }
    if pos < 0 {
        return Err(0);
    }
    if pos > len {
        return Err(len as usize);
    }
    return Ok(pos as usize);
}

impl Value {
    pub fn is_nil(&self) -> bool {
        match self {
            &Value::Nil => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            &Value::Data(_) => true,
            &Value::Integer(_) => true,
            _ => false,
        }
    }

    pub fn is_list(&self) -> bool {
        match self {
            &Value::List(_) => true,
            _ => false,
        }
    }

    pub fn is_set(&self) -> bool {
        match self {
            &Value::Set(_) => true,
            _ => false,
        }
    }

    pub fn set(&mut self, value: Vec<u8>) -> Result<(), OperationError> {
        if value.len() < 32 { // ought to be enough!
            if let Ok(utf8) = from_utf8(&*value) {
                if let Ok(i) = utf8.parse::<i64>() {
                    *self = Value::Integer(i);
                    return Ok(());
                }
            }
        }
        *self = Value::Data(value);
        return Ok(());
    }

    pub fn append(&mut self, value: Vec<u8>) -> Result<usize, OperationError> {
        match self {
            &mut Value::Nil => {
                let len = value.len();
                *self = Value::Data(value);
                return Ok(len);
            },
            &mut Value::Data(ref mut data) => { data.extend(value); return Ok(data.len()); },
            &mut Value::Integer(i) => {
                let oldstr = format!("{}", i);
                let len = oldstr.len() + value.len();
                *self = Value::Data([oldstr.into_bytes(), value].concat());
                return Ok(len);
            },
            _ => return Err(OperationError::WrongTypeError),
        }
    }

    pub fn incr(&mut self, incr: i64) -> Result<i64, OperationError> {
        let mut newval:i64;
        match self {
            &mut Value::Nil => {
                newval = incr;
            },
            &mut Value::Integer(i) => {
                let tmp_newval = i.checked_add(incr);
                match tmp_newval {
                    Some(v) => newval = v,
                    None => return Err(OperationError::OverflowError),
                }
            },
            &mut Value::Data(ref data) => {
                if data.len() > 32 {
                    return Err(OperationError::OverflowError);
                }
                let res = try!(from_utf8(&data));
                let ival = try!(res.parse::<i64>());
                let tmp_newval = ival.checked_add(incr);
                match tmp_newval {
                    Some(v) => newval = v,
                    None => return Err(OperationError::OverflowError),
                }
            },
            _ => return Err(OperationError::WrongTypeError),
        }
        *self = Value::Integer(newval);
        return Ok(newval);
    }

    pub fn getrange(&self, _start: i64, _stop: i64) -> Result<Vec<u8>, OperationError> {
        let s = match self {
            &Value::Nil => return Ok(Vec::new()),
            &Value::Integer(ref i) => format!("{}", i).into_bytes(),
            &Value::Data(ref s) => s.clone(),
            _ => return Err(OperationError::WrongTypeError),
        };

        let len = s.len();
        let start = match normalize_position(_start, len) {
            Ok(i) => i,
            Err(i) => if i == 0 { 0 } else { return Ok(Vec::new()); }
        } as usize;
        let stop = match normalize_position(_stop, len) {
            Ok(i) => i,
            Err(i) => if i == 0 { return Ok(Vec::new()); } else { len }
        } as usize;
        let mut v = Vec::with_capacity(stop - start + 1);
        v.extend(s[start..stop + 1].iter());
        Ok(v)
    }

    pub fn setrange(&mut self, _index: i64, data: Vec<u8>) -> Result<usize, OperationError> {
        match self {
            &mut Value::Nil => *self = Value::Data(Vec::new()),
            &mut Value::Integer(i) => *self = Value::Data(format!("{}", i).into_bytes()),
            &mut Value::Data(_) => (),
            _ => return Err(OperationError::WrongTypeError),
        };
        let mut d = match self {
            &mut Value::Data(ref mut d) => d,
            _ => panic!("Value should be data"),
        };
        let mut index = match normalize_position(_index, d.len()) {
            Ok(i) => i,
            Err(p) => if p == 0 { p } else { _index as usize },
        };
        for _ in d.len()..index {
            d.push(0);
        }
        for c in data {
            d.push(c);
            if index < d.len() - 1 {
                d.swap_remove(index);
            }
            index += 1;
        }
        Ok(d.len())
    }

    pub fn push(&mut self, el: Vec<u8>, right: bool) -> Result<usize, OperationError> {
        let listsize;
        match self {
            &mut Value::Nil => {
                let mut list = LinkedList::new();
                list.push_back(el);
                *self = Value::List(list);
                listsize = 1;
            },
            &mut Value::List(ref mut list) => {
                if right {
                    list.push_back(el);
                } else {
                    list.push_front(el);
                }
                listsize = list.len();
            }
            _ => return Err(OperationError::WrongTypeError),
        }
        return Ok(listsize);
    }

    pub fn pop(&mut self, right: bool) -> Result<Option<Vec<u8>>, OperationError> {
        let el;
        let mut clear;
        match self {
            &mut Value::Nil => {
                return Ok(None);
            },
            &mut Value::List(ref mut list) => {
                if right {
                    el = list.pop_back();
                } else {
                    el = list.pop_front();
                }
                clear = list.len() == 0;
            }
            _ => return Err(OperationError::WrongTypeError),
        }
        if clear {
            *self = Value::Nil;
        }
        return Ok(el);
    }

    pub fn lindex(&self, _index: i64) -> Result<Option<&Vec<u8>>, OperationError> {
        return match self {
            &Value::List(ref list) => {
                let index = match normalize_position(_index, list.len()) {
                    Ok(i) => i,
                    Err(_) => return Ok(None),
                };
                return Ok(list.iter().nth(index as usize));
            },
            _ => Err(OperationError::WrongTypeError),
        }
    }

    pub fn linsert(&mut self, before: bool, pivot: Vec<u8>, value: Vec<u8>) -> Result<Option<usize>, OperationError> {
        match self {
            &mut Value::List(ref mut list) => {
                let pos;
                match list.iter().position(|x| x == &pivot) {
                    Some(_pos) => {
                        if before {
                            pos = _pos;
                        } else {
                            pos = _pos + 1;
                        }
                    },
                    None => return Ok(None),
                }
                let mut right = list.split_off(pos);
                list.push_back(value);
                list.append(&mut right);
                return Ok(Some(list.len()));
            },
            _ => return Err(OperationError::WrongTypeError),
        };
    }

    pub fn llen(&self) -> Result<usize, OperationError> {
        return match self {
            &Value::List(ref list) => Ok(list.len()),
            &Value::Nil => Ok(0),
            _ => Err(OperationError::WrongTypeError),
        };
    }

    pub fn lrange(&self, _start: i64, _stop: i64) -> Result<Vec<&Vec<u8>>, OperationError> {
        match self {
            &Value::List(ref list) => {
                let len = list.len();
                let start = match normalize_position(_start, len) {
                    Ok(i) => i,
                    Err(i) => if i == 0 { 0 } else { return Ok(Vec::new()); },
                };
                let stop = match normalize_position(_stop, len) {
                    Ok(i) => i,
                    Err(i) => if i == 0 { return Ok(Vec::new()); } else { i },
                };
                return Ok(list.iter().skip(start as usize).take(stop as usize - start as usize + 1).collect());
            },
            _ => return Err(OperationError::WrongTypeError),
        };
    }

    pub fn lrem(&mut self, left: bool, limit: usize, value: Vec<u8>) -> Result<usize, OperationError> {
        let mut count = 0;
        let mut newlist = LinkedList::new();
        match self {
            &mut Value::List(ref mut list) => {
                if left {
                    while limit == 0 || count < limit {
                        match list.pop_front() {
                            None => break,
                            Some(el) => {
                                if el != value {
                                    newlist.push_back(el);
                                } else {
                                    count += 1;
                                }
                            }
                        }
                    }
                    newlist.append(list);
                } else {
                    while limit == 0 || count < limit {
                        match list.pop_back() {
                            None => break,
                            Some(el) => {
                                if el != value {
                                    newlist.push_front(el);
                                } else {
                                    count += 1;
                                }
                            }
                        }
                    }
                    // omg, ugly code, let me explain
                    // append will merge right two lists and clear the parameter
                    // newlist is the one that will survive after lrem
                    // but list needs to be at the beginning, so we are merging
                    // first to list and then to newlist
                    list.append(&mut newlist);
                    newlist.append(list);
                }
            },
            _ => return Err(OperationError::WrongTypeError),
        };
        if newlist.len() == 0 {
            *self = Value::Nil;
        } else {
            *self = Value::List(newlist);
        }
        return Ok(count);
    }

    pub fn lset(&mut self, index: i64, value: Vec<u8>) -> Result<(), OperationError> {
        return match self {
            &mut Value::List(ref mut list) => {
                let i = match normalize_position(index, list.len()) {
                    Ok(i) => i,
                    Err(_) => return Err(OperationError::OutOfBoundsError),
                };
                // this unwrap is safe because `i` is already validated to be inside the list
                let el = list.iter_mut().skip(i).next().unwrap();
                *el = value;
                return Ok(());
            },
            _ => return Err(OperationError::WrongTypeError),
        }
    }

    pub fn ltrim(&mut self, _start: i64, _stop: i64) -> Result<(), OperationError> {
        let mut newlist;
        match self {
            &mut Value::List(ref mut list) => {
                let len = list.len();
                let start = match normalize_position(_start, len) {
                    Ok(i) => i,
                    Err(i) => if i == 0 { 0 } else {
                        list.split_off(len);
                        len
                    },
                };
                let stop = match normalize_position(_stop, len) {
                    Ok(i) => i,
                    Err(i) => if i == 0 {
                        list.split_off(len);
                        0
                    } else { i },
                };
                list.split_off(stop + 1);
                newlist = list.split_off(start);
            },
            _ => return Err(OperationError::WrongTypeError),
        }
        *self = Value::List(newlist);
        return Ok(());
    }

    pub fn sadd(&mut self, el: Vec<u8>) -> Result<bool, OperationError> {
        match self {
            &mut Value::Nil => {
                let mut set = HashSet::new();
                set.insert(el);
                *self = Value::Set(set);
                Ok(true)
            },
            &mut Value::Set(ref mut set) => Ok(set.insert(el)),
            _ => Err(OperationError::WrongTypeError),
        }
    }

    pub fn srem(&mut self, el: &Vec<u8>) -> Result<bool, OperationError> {
        match self {
            &mut Value::Nil => Ok(false),
            &mut Value::Set(ref mut set) => Ok(set.remove(el)),
            _ => Err(OperationError::WrongTypeError),
        }
    }

    pub fn sismember(&self, el: &Vec<u8>) -> Result<bool, OperationError> {
        match self {
            &Value::Nil => Ok(false),
            &Value::Set(ref set) => Ok(set.contains(el)),
            _ => Err(OperationError::WrongTypeError),
        }
    }

    pub fn scard(&self) -> Result<usize, OperationError> {
        match self {
            &Value::Nil => Ok(0),
            &Value::Set(ref set) => Ok(set.len()),
            _ => Err(OperationError::WrongTypeError),
        }
    }

    pub fn srandmember(&self, count: usize, allow_duplicates: bool) -> Result<Vec<Vec<u8>>, OperationError> {
        // TODO: implemented in O(n), should be O(1)
        let set = match self {
            &Value::Nil => return Ok(Vec::new()),
            &Value::Set(ref s) => s,
            _ => return Err(OperationError::WrongTypeError),
        };

        if allow_duplicates {
            let mut r = Vec::new();
            for _ in 0..count {
                let pos = random::<usize>() % set.len();
                r.push(set.iter().skip(pos).take(1).next().unwrap().clone());
            }
            return Ok(r);
        } else {
            if count >= set.len() {
                return Ok(set.iter().map(|x| x.clone()).collect::<Vec<_>>());
            }
            let mut s = HashSet::new();
            while s.len() < count {
                let pos = random::<usize>() % set.len();
                s.insert(set.iter().skip(pos).take(1).next().unwrap().clone());
            }
            return Ok(s.iter().map(|x| x.clone()).collect::<Vec<_>>());
        }
    }

    pub fn spop(&mut self, count: usize) -> Result<Vec<Vec<u8>>, OperationError> {
        // TODO: implemented in O(n), should be O(1)

        let len = try!(self.scard());
        if count >= len {
            let r = {
                let set = match self {
                    &mut Value::Nil => return Ok(Vec::new()),
                    &mut Value::Set(ref mut s) => s,
                    _ => return Err(OperationError::WrongTypeError),
                };
                set.iter().map(|x| x.clone()).collect::<Vec<_>>()
            };
            *self = Value::Nil;
            return Ok(r);
        }

        let r = try!(self.srandmember(count, false));

        let mut set = match self {
            &mut Value::Nil => return Ok(Vec::new()),
            &mut Value::Set(ref mut s) => s,
            _ => return Err(OperationError::WrongTypeError),
        };

        for member in r.iter() {
            set.remove(member);
        }

        Ok(r)
    }

    pub fn sdiff(&self, sets: &Vec<&Value>) -> Result<HashSet<Vec<u8>>, OperationError> {
        match self {
            &Value::Nil => Ok(HashSet::new()),
            &Value::Set(ref original_set) => {
                let mut elements: HashSet<Vec<u8>> = original_set.clone();
                for value in sets {
                    match value {
                        &&Value::Nil => {},
                        &&Value::Set(ref set) => {
                            for el in set {
                                elements.remove(el);
                            }
                        },
                        _ => return Err(OperationError::WrongTypeError),
                    }
                }
                Ok(elements)
            },
            _ => Err(OperationError::WrongTypeError),
        }
    }

    pub fn create_set(&mut self, set: HashSet<Vec<u8>>) {
        *self = Value::Set(set);
    }

    pub fn zadd(&mut self, s: f64, el: Vec<u8>, nx: bool, xx: bool, ch: bool) -> Result<bool, OperationError> {
        match self {
            &mut Value::Nil => {
                if xx {
                    return Ok(false);
                }
                let mut zset = SkipMap::new();
                let mut hmap = HashMap::new();
                zset.insert(Float::new(s), el.clone());
                hmap.insert(el, Float::new(s));
                *self = Value::SortedSet(zset, hmap);
                Ok(true)
            },
            &mut Value::SortedSet(ref mut zset, ref mut hmap) => {
                let contains = hmap.contains_key(&el);
                if contains && nx {
                    return Ok(false);
                }
                if !contains && xx {
                    return Ok(false);
                }
                if contains {
                    let val = hmap.get(&el).unwrap().get();
                    if ch && val == &s {
                        return Ok(false);
                    }
                }
                zset.insert(Float::new(s), el.clone());
                hmap.insert(el, Float::new(s));
                if ch {
                    Ok(true)
                } else {
                    Ok(!contains)
                }
            },
            _ => Err(OperationError::WrongTypeError),
        }
    }
}

pub struct Database {
    data: Vec<HashMap<Vec<u8>, Value>>,
    data_expiration_ns: Vec<HashMap<Vec<u8>, i64>>,
    pub size: usize,
    subscribers: HashMap<Vec<u8>, HashMap<usize, Sender<PubsubEvent>>>,
    pattern_subscribers: HashMap<Vec<u8>, HashMap<usize, Sender<PubsubEvent>>>,
    key_subscribers: HashMap<Vec<u8>, HashMap<usize, Sender<bool>>>,
    subscriber_id: usize,
}

fn create_database(size: usize) -> Database {
    let mut data = Vec::with_capacity(size);
    let mut data_expiration_ns = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(HashMap::new());
        data_expiration_ns.push(HashMap::new());
    }
    return Database {
        data: data,
        data_expiration_ns: data_expiration_ns,
        size: size,
        subscribers: HashMap::new(),
        pattern_subscribers: HashMap::new(),
        key_subscribers: HashMap::new(),
        subscriber_id: 0,
    };
}

impl Database {
    pub fn mock() -> Database {
        create_database(16)
    }

    pub fn new(config: &Config) -> Database {
        create_database(config.databases as usize)
    }

    fn is_expired(&self, index: usize, key: &Vec<u8>) -> bool {
        match self.data_expiration_ns[index].get(key) {
            Some(t) => t < &mstime(),
            None => false,
        }
    }

    pub fn get(&self, index: usize, key: &Vec<u8>) -> Option<&Value> {
        if self.is_expired(index, key) {
            None
        } else {
            self.data[index].get(key)
        }
    }

    pub fn get_mut(&mut self, index: usize, key: &Vec<u8>) -> Option<&mut Value> {
        if self.is_expired(index, key) {
            self.remove(index, key);
            None
        } else {
            self.data[index].get_mut(key)
        }
    }

    pub fn remove(&mut self, index: usize, key: &Vec<u8>) -> Option<Value> {
        let mut r = self.data[index].remove(key);
        if self.is_expired(index, key) {
            r = None;
        }
        self.data_expiration_ns[index].remove(key);
        r
    }

    pub fn set_msexpiration(&mut self, index: usize, key: Vec<u8>, msexpiration: i64) {
        self.data_expiration_ns[index].insert(key, msexpiration);
    }

    pub fn get_msexpiration(&mut self, index: usize, key: &Vec<u8>) -> Option<&i64> {
        self.data_expiration_ns[index].get(key)
    }

    pub fn remove_msexpiration(&mut self, index: usize, key: &Vec<u8>) -> Option<i64> {
        self.data_expiration_ns[index].remove(key)
    }

    pub fn clear(&mut self, index: usize) {
        self.data[index].clear()
    }

    pub fn get_or_create(&mut self, index: usize, key: &Vec<u8>) -> &mut Value {
        if self.get(index, key).is_some() {
            return self.get_mut(index, key).unwrap();
        }
        let val = Value::Nil;
        self.data[index].insert(Vec::clone(key), val);
        return self.data[index].get_mut(key).unwrap();
    }

    fn ensure_key_subscribers(&mut self, key: &Vec<u8>) {
        if !self.key_subscribers.contains_key(key) {
            self.key_subscribers.insert(key.clone(), HashMap::new());
        }
    }

    pub fn key_subscribe(&mut self, key: &Vec<u8>, sender: Sender<bool>) -> usize {
        self.ensure_key_subscribers(key);
        let mut key_subscribers = self.key_subscribers.get_mut(key).unwrap();
        let subscriber_id = self.subscriber_id;
        key_subscribers.insert(subscriber_id, sender);
        self.subscriber_id += 1;
        subscriber_id
    }

    pub fn key_publish(&mut self, key: &Vec<u8>) {
        let mut torem = Vec::new();
        match self.key_subscribers.get_mut(key) {
            Some(mut channels) => {
                for (subscriber_id, channel) in channels.iter() {
                    match channel.send(true) {
                        Ok(_) => (),
                        Err(_) => { torem.push(subscriber_id.clone()); () },
                    }
                }
                for subscriber_id in torem {
                    channels.remove(&subscriber_id);
                }
            }
            None => (),
        }
    }

    fn ensure_channel(&mut self, channel: &Vec<u8>) {
        if !self.subscribers.contains_key(channel) {
            self.subscribers.insert(channel.clone(), HashMap::new());
        }
    }

    pub fn subscribe(&mut self, channel: Vec<u8>, sender: Sender<PubsubEvent>) -> usize {
        self.ensure_channel(&channel);
        let mut channelsubscribers = self.subscribers.get_mut(&channel).unwrap();
        let subscriber_id = self.subscriber_id;
        channelsubscribers.insert(subscriber_id, sender);
        self.subscriber_id += 1;
        subscriber_id
    }

    pub fn unsubscribe(&mut self, channel: Vec<u8>, subscriber_id: usize) -> bool {
        if !self.subscribers.contains_key(&channel) {
            return false;
        }
        let mut channelsubscribers = self.subscribers.get_mut(&channel).unwrap();
        channelsubscribers.remove(&subscriber_id).is_some()
    }

    fn pensure_channel(&mut self, pattern: &Vec<u8>) {
        if !self.pattern_subscribers.contains_key(pattern) {
            self.pattern_subscribers.insert(pattern.clone(), HashMap::new());
        }
    }

    pub fn psubscribe(&mut self, pattern: Vec<u8>, sender: Sender<PubsubEvent>) -> usize {
        self.pensure_channel(&pattern);
        let mut channelsubscribers = self.pattern_subscribers.get_mut(&pattern).unwrap();
        let subscriber_id = self.subscriber_id;
        channelsubscribers.insert(subscriber_id, sender);
        self.subscriber_id += 1;
        subscriber_id
    }

    pub fn punsubscribe(&mut self, pattern: Vec<u8>, subscriber_id: usize) -> bool {
        if !self.pattern_subscribers.contains_key(&pattern) {
            return false;
        }
        let mut channelsubscribers = self.pattern_subscribers.get_mut(&pattern).unwrap();
        channelsubscribers.remove(&subscriber_id).is_some()
    }

    pub fn publish(&self, channel_name: &Vec<u8>, message: &Vec<u8>) -> usize {
        let mut c = 0;
        match self.subscribers.get(channel_name) {
            Some(channels) => {
                for (_, channel) in channels {
                    match channel.send(PubsubEvent::Message(channel_name.clone(), None, message.clone())) {
                        Ok(_) => c += 1,
                        Err(_) => (),
                    }
                }
            }
            None => (),
        }
        for (pattern, channels) in self.pattern_subscribers.iter() {
            if glob_match(&pattern, &channel_name, false) {
                for (_, channel) in channels {
                    match channel.send(PubsubEvent::Message(channel_name.clone(), Some(pattern.clone()), message.clone())) {
                        Ok(_) => c += 1,
                        Err(_) => (),
                    }
                }
            }
        }
        c
    }

    pub fn clearall(&mut self) {
        for index in 0..self.size {
            self.data[index].clear();
        }
    }
}
