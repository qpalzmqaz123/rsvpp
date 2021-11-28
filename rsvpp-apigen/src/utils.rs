pub trait Capitalize {
    fn capitalize(&self) -> Self;
    fn hump(&self) -> Self;
}

impl Capitalize for String {
    fn capitalize(&self) -> Self {
        if self.len() > 0 {
            let first = self.chars().nth(0).unwrap();
            let first = first.to_uppercase().next().unwrap();

            format!("{}{}", first, &self[1..])
        } else {
            self.clone()
        }
    }

    fn hump(&self) -> Self {
        let strs = self.split('_').collect::<Vec<_>>();
        let mut ret = String::new();

        for s in strs {
            let cap = s.to_string().capitalize();
            ret.push_str(cap.as_str());
        }

        ret
    }
}
