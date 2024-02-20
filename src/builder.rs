use bevy::math::{Affine2, Vec2};
use lyon::path::builder::NoAttributes;
use lyon::path::traits::Build;
pub use lyon::path::Winding;
use lyon::tessellation::*;

pub struct Builder<'a> {
    builder: NoAttributes<FillBuilder<'a>>,
    transform: Affine2,
    stack: Vec<Affine2>,
}

#[inline]
fn vec2p(v: Vec2) -> lyon::math::Point {
    lyon::math::Point::new(v.x, v.y)
}

#[inline]
fn vec2v(v: Vec2) -> lyon::math::Vector {
    lyon::math::Vector::new(v.x, v.y)
}

impl<'a> Builder<'a> {
    pub fn new(builder: NoAttributes<FillBuilder<'a>>) -> Self {
        Builder {
            builder,
            transform: Affine2::IDENTITY,
            stack: Vec::new(),
        }
    }

    // apply the transform to the builder next to the existing ones
    pub fn transform(&mut self, transform: Affine2) -> &mut Self {
        self.transform *= transform;
        self
    }

    pub fn push(&mut self) -> &mut Self {
        self.stack.push(self.transform);
        self
    }

    pub fn pop(&mut self) -> &mut Self {
        self.transform = self.stack.pop().unwrap();
        self
    }

    pub fn rotate(&mut self, angle: f32) -> &mut Self {
        self.transform = self.transform * Affine2::from_angle(angle);
        self
    }

    pub fn translate(&mut self, t: Vec2) -> &mut Self {
        self.transform = self.transform * Affine2::from_translation(t);
        self
    }

    pub fn scale(&mut self, s: Vec2) -> &mut Self {
        self.transform = self.transform * Affine2::from_scale(s);
        self
    }

    pub fn scale_uniform(&mut self, s: f32) -> &mut Self {
        self.scale(Vec2::new(s, s))
    }

    pub fn begin(&mut self, point: Vec2) -> &mut Self {
        self.builder
            .begin(vec2p(self.transform.transform_point2(point)));
        self
    }

    pub fn begin_here(&mut self) -> &mut Self {
        self.begin(Vec2::ZERO)
    }

    pub fn begin_push(&mut self, translate: Vec2) -> &mut Self {
        self.push().translate(translate).begin(Vec2::ZERO)
    }

    pub fn line_to(&mut self, point: Vec2) -> &mut Self {
        self.builder
            .line_to(vec2p(self.transform.transform_point2(point)));
        self
    }

    pub fn end(&mut self, close: bool) -> &mut Self {
        self.builder.end(close);
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.end(true)
    }

    pub fn close_pop(&mut self) -> &mut Self {
        self.close().pop()
    }

    pub fn end_pop(&mut self, close: bool) -> &mut Self {
        self.end(close).pop()
    }

    /// Builds a path object, consuming the builder.
    #[inline]
    pub fn build<P>(self) -> P
    where
        FillBuilder<'a>: Build<PathType = P>,
    {
        self.builder.build()
    }

    pub fn quadratic_bezier_to(&mut self, ctrl: Vec2, to: Vec2) -> &mut Self {
        self.builder.quadratic_bezier_to(
            vec2p(self.transform.transform_point2(ctrl)),
            vec2p(self.transform.transform_point2(to)),
        );
        self
    }

    pub fn cubic_bezier_to(&mut self, ctrl1: Vec2, ctrl2: Vec2, to: Vec2) -> &mut Self {
        self.builder.cubic_bezier_to(
            vec2p(self.transform.transform_point2(ctrl1)),
            vec2p(self.transform.transform_point2(ctrl2)),
            vec2p(self.transform.transform_point2(to)),
        );
        self
    }

    pub fn add_rectangle(&mut self, rect: bevy::math::Rect, winding: Winding) -> &mut Self {
        let p0 = vec2p(self.transform.transform_point2(rect.min));
        let p1 = vec2p(self.transform.transform_point2(rect.max));
        self.builder
            .add_rectangle(&lyon::math::Box2D::new(p0, p1), winding);
        self
    }

    pub fn add_circle(&mut self, center: Vec2, radius: f32, winding: Winding) -> &mut Self {
        self.builder.add_circle(
            vec2p(self.transform.transform_point2(center)),
            radius,
            winding,
        );
        self
    }

    pub fn add_ellipse(
        &mut self,
        center: Vec2,
        radii: Vec2,
        x_rotation: f32,
        winding: Winding,
    ) -> &mut Self {
        self.builder.add_ellipse(
            vec2p(self.transform.transform_point2(center)),
            vec2v(self.transform.transform_vector2(radii)),
            lyon::math::Angle::radians(x_rotation),
            winding,
        );
        self
    }
}
