use std::collections::BTreeMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<JsonValue>),
    Obj(BTreeMap<String, JsonValue>),
}

pub fn parse(input: &str) -> Result<JsonValue, String> {
    let mut p = Parser {
        b: input.as_bytes(),
        i: 0,
    };
    let v = p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return Err("trailing json".to_string());
    }
    Ok(v)
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}
impl<'a> Parser<'a> {
    fn value(&mut self) -> Result<JsonValue, String> {
        self.ws();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(JsonValue::Str),
            Some(b't') | Some(b'f') => self.bool(),
            Some(b'n') => {
                self.token(b"null")?;
                Ok(JsonValue::Null)
            }
            Some(b'-') | Some(b'0'..=b'9') => self.number(),
            _ => Err("unexpected token".to_string()),
        }
    }
    fn object(&mut self) -> Result<JsonValue, String> {
        self.expect(b'{')?;
        let mut m = BTreeMap::new();
        loop {
            self.ws();
            if self.peek() == Some(b'}') {
                self.i += 1;
                break;
            }
            let k = self.string()?;
            self.ws();
            self.expect(b':')?;
            let v = self.value()?;
            m.insert(k, v);
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    break;
                }
                _ => return Err("bad object".to_string()),
            }
        }
        Ok(JsonValue::Obj(m))
    }
    fn array(&mut self) -> Result<JsonValue, String> {
        self.expect(b'[')?;
        let mut a = Vec::new();
        loop {
            self.ws();
            if self.peek() == Some(b']') {
                self.i += 1;
                break;
            }
            a.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    break;
                }
                _ => return Err("bad array".to_string()),
            }
        }
        Ok(JsonValue::Arr(a))
    }
    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut s = String::new();
        while let Some(c) = self.peek() {
            self.i += 1;
            match c {
                b'"' => return Ok(s),
                b'\\' => {
                    let e = self.peek().ok_or("bad escape")?;
                    self.i += 1;
                    s.push(match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        _ => return Err("unsupported escape".to_string()),
                    });
                }
                _ => s.push(c as char),
            }
        }
        Err("unterminated string".to_string())
    }
    fn bool(&mut self) -> Result<JsonValue, String> {
        if self.match_token(b"true") {
            Ok(JsonValue::Bool(true))
        } else if self.match_token(b"false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("bad bool".to_string())
        }
    }
    fn number(&mut self) -> Result<JsonValue, String> {
        let st = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.i += 1;
        }
        if self.peek() == Some(b'.') {
            self.i += 1;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.i += 1;
            }
        }
        let n: f64 = std::str::from_utf8(&self.b[st..self.i])
            .map_err(|e| e.to_string())?
            .parse()
            .map_err(|_| "bad num".to_string())?;
        Ok(JsonValue::Num(n))
    }
    fn ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.i += 1;
        }
    }
    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }
    fn expect(&mut self, c: u8) -> Result<(), String> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err("unexpected char".to_string())
        }
    }
    fn token(&mut self, t: &[u8]) -> Result<(), String> {
        if self.match_token(t) {
            Ok(())
        } else {
            Err("unexpected token".to_string())
        }
    }
    fn match_token(&mut self, t: &[u8]) -> bool {
        if self.b.get(self.i..self.i + t.len()) == Some(t) {
            self.i += t.len();
            true
        } else {
            false
        }
    }
}

pub fn obj(v: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    if let JsonValue::Obj(m) = v {
        Some(m)
    } else {
        None
    }
}
pub fn arr(v: &JsonValue) -> Option<&Vec<JsonValue>> {
    if let JsonValue::Arr(a) = v {
        Some(a)
    } else {
        None
    }
}
pub fn strv(v: &JsonValue) -> Option<&str> {
    if let JsonValue::Str(s) = v {
        Some(s)
    } else {
        None
    }
}
