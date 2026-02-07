use std::borrow::Cow;
use web_sys::CssStyleDeclaration;

#[derive(Clone, Debug, PartialEq)]
pub enum FluidValue {
    Number(f64),
    Text(Cow<'static, str>),
}

impl FluidValue {
    fn to_string_value(&self) -> String {
        match self {
            FluidValue::Number(value) => value.to_string(),
            FluidValue::Text(value) => value.to_string(),
        }
    }
}

impl From<f64> for FluidValue {
    fn from(value: f64) -> Self {
        FluidValue::Number(value)
    }
}

impl From<f32> for FluidValue {
    fn from(value: f32) -> Self {
        FluidValue::Number(value as f64)
    }
}

impl From<i32> for FluidValue {
    fn from(value: i32) -> Self {
        FluidValue::Number(value as f64)
    }
}

impl From<u32> for FluidValue {
    fn from(value: u32) -> Self {
        FluidValue::Number(value as f64)
    }
}

impl From<&'static str> for FluidValue {
    fn from(value: &'static str) -> Self {
        FluidValue::Text(Cow::Borrowed(value))
    }
}

impl From<String> for FluidValue {
    fn from(value: String) -> Self {
        FluidValue::Text(Cow::Owned(value))
    }
}

impl From<Cow<'static, str>> for FluidValue {
    fn from(value: Cow<'static, str>) -> Self {
        FluidValue::Text(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transform {
    translate_x: Option<f64>,
    translate_y: Option<f64>,
    scale: Option<f64>,
    rotate: Option<f64>,
}

impl Transform {
    fn is_empty(&self) -> bool {
        self.translate_x.is_none()
            && self.translate_y.is_none()
            && self.scale.is_none()
            && self.rotate.is_none()
    }

    fn to_css(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        let mut parts = Vec::with_capacity(4);
        if self.translate_x.is_some() || self.translate_y.is_some() {
            let x = self.translate_x.unwrap_or(0.0);
            let y = self.translate_y.unwrap_or(0.0);
            parts.push(format!("translate3d({}px, {}px, 0px)", x, y));
        }
        if let Some(scale) = self.scale {
            parts.push(format!("scale({})", scale));
        }
        if let Some(rotate) = self.rotate {
            parts.push(format!("rotate({}deg)", rotate));
        }

        Some(parts.join(" "))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FluidStyle {
    props: Vec<(Cow<'static, str>, FluidValue)>,
    transform: Transform,
}

impl FluidStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.props.is_empty() && self.transform.is_empty()
    }

    pub fn set<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<FluidValue>,
    {
        self.props.push((key.into(), value.into()));
        self
    }

    pub fn with<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<FluidValue>,
    {
        self.set(key, value);
        self
    }

    pub fn opacity(mut self, value: f64) -> Self {
        self.set("opacity", value);
        self
    }

    pub fn width(mut self, px: f64) -> Self {
        self.set("width", format!("{}px", px));
        self
    }

    pub fn height(mut self, px: f64) -> Self {
        self.set("height", format!("{}px", px));
        self
    }

    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.set("width", format!("{}px", width));
        self.set("height", format!("{}px", height));
        self
    }

    pub fn x(mut self, px: f64) -> Self {
        self.transform.translate_x = Some(px);
        self
    }

    pub fn y(mut self, px: f64) -> Self {
        self.transform.translate_y = Some(px);
        self
    }

    pub fn translate_x(mut self, px: f64) -> Self {
        self.transform.translate_x = Some(px);
        self
    }

    pub fn translate_y(mut self, px: f64) -> Self {
        self.transform.translate_y = Some(px);
        self
    }

    pub fn scale(mut self, value: f64) -> Self {
        self.transform.scale = Some(value);
        self
    }

    pub fn rotate(mut self, deg: f64) -> Self {
        self.transform.rotate = Some(deg);
        self
    }

    pub fn to_props(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut props = Vec::with_capacity(self.props.len() + 1);
        for (key, value) in &self.props {
            props.push((key.clone(), value.to_string_value()));
        }

        if !self
            .props
            .iter()
            .any(|(key, _)| key.as_ref() == "transform")
            && let Some(transform) = self.transform.to_css()
        {
            props.push((Cow::Borrowed("transform"), transform));
        }

        props
    }

    pub(crate) fn apply_to(&self, style: &CssStyleDeclaration) {
        for (key, value) in &self.props {
            let _ = style.set_property(key.as_ref(), &value.to_string_value());
        }

        if self
            .props
            .iter()
            .all(|(key, _)| key.as_ref() != "transform")
            && let Some(transform) = self.transform.to_css()
        {
            let _ = style.set_property("transform", &transform);
        }
    }
}

#[macro_export]
macro_rules! style {
    ($($key:literal => $value:expr),* $(,)?) => {{
        let mut style = $crate::FluidStyle::new();
        $(
            style.set($key, $value);
        )*
        style
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_transform_chain() {
        let style = FluidStyle::new()
            .translate_x(12.0)
            .translate_y(-4.0)
            .scale(1.1)
            .rotate(45.0);
        let props = style.to_props();
        let transform = props
            .iter()
            .find(|(key, _)| key.as_ref() == "transform")
            .map(|(_, value)| value.as_str())
            .unwrap();
        assert_eq!(
            transform,
            "translate3d(12px, -4px, 0px) scale(1.1) rotate(45deg)"
        );
    }

    #[test]
    fn style_macro_sets_props() {
        let style = style!("opacity" => 0.4, "filter" => "blur(4px)");
        let props = style.to_props();
        assert!(
            props
                .iter()
                .any(|(key, value)| key.as_ref() == "opacity" && value == "0.4")
        );
        assert!(
            props
                .iter()
                .any(|(key, value)| key.as_ref() == "filter" && value == "blur(4px)")
        );
    }
}
