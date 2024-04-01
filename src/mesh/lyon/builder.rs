use bevy::math::{Affine2, Vec2};
pub use lyon::path::traits::PathBuilder;
use lyon::path::{builder::NoAttributes, traits::Build, Winding};

/// This structure wraps a `lyon::tesselation::PathBuilder` and adds functionality to apply transformations to the path being built.
pub struct PBuilder<T>
where
    T: PathBuilder,
{
    builder: NoAttributes<T>,
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

impl<T> PBuilder<T>
where
    T: PathBuilder,
{
    /// Creates a new builder with the given fill builder.
    pub fn new(builder: NoAttributes<T>) -> Self {
        PBuilder {
            builder,
            transform: Affine2::IDENTITY,
            stack: Vec::new(),
        }
    }

    /// Stores the current transformation on the stack.
    pub fn push(&mut self) -> &mut Self {
        self.stack.push(self.transform);
        self
    }

    /// Restores the transformation from the top of the stack and replace the current transformation with it.
    pub fn pop(&mut self) -> &mut Self {
        self.transform = self.stack.pop().unwrap();
        self
    }

    /// Applies a transformation to the path being built.
    pub fn transform(&mut self, transform: Affine2) -> &mut Self {
        self.transform *= transform;
        self
    }

    /// Overwrites the current transformation with the given one.
    pub fn set_transform(&mut self, transform: Affine2) -> &mut Self {
        self.transform = transform;
        self
    }

    /// Applies an additional rotation to the next points and vectors.  
    pub fn rotate(&mut self, angle: f32) -> &mut Self {
        self.transform = self.transform * Affine2::from_angle(angle);
        self
    }

    /// Applies an additional translation to the next points (not vectors!).
    pub fn translate(&mut self, t: Vec2) -> &mut Self {
        self.transform = self.transform * Affine2::from_translation(t);
        self
    }

    /// Applies an additional scaling to the next points and vectors.
    pub fn scale(&mut self, s: Vec2) -> &mut Self {
        self.transform = self.transform * Affine2::from_scale(s);
        self
    }

    /// Applies an additional uniform scaling to the next points and vectors.
    pub fn scale_uniform(&mut self, s: f32) -> &mut Self {
        self.scale(Vec2::new(s, s))
    }

    /// Begins a new sub-path at the given point.
    pub fn begin(&mut self, point: Vec2) -> &mut Self {
        self.builder
            .begin(vec2p(self.transform.transform_point2(point)));
        self
    }

    /// Begins a new sub-path at the origin of the current transformation.
    pub fn begin_here(&mut self) -> &mut Self {
        self.begin(Vec2::ZERO)
    }

    /// Pushes the current transformation onto the stack, translates to the given position, and begins a new sub-path at the origin of the new transformation.
    ///
    /// This is useful for creating a sub-path that is in a different coordinate system than the current path and you want to continue afterwards.
    pub fn begin_trans_push(&mut self, translate: Vec2) -> &mut Self {
        self.push().translate(translate).begin(Vec2::ZERO)
    }

    /// Ends the current sub path.
    ///
    /// A sub-path must be in progress when this method is called.
    /// After this method is called, there is no sub-path in progress until begin is called again.
    pub fn end(&mut self, close: bool) -> &mut Self {
        self.builder.end(close);
        self
    }

    /// Ends the current sub path and closes it.
    ///
    /// A sub-path must be in progress when this method is called.
    /// After this method is called, there is no sub-path in progress until begin is called again.
    pub fn close(&mut self) -> &mut Self {
        self.end(true)
    }

    /// Ends the current sub path, closes it, and restores the previous transformation from the stack.
    pub fn close_pop(&mut self) -> &mut Self {
        self.close().pop()
    }

    /// Ends the current sub path without closing it and restores the previous transformation from the stack.
    pub fn end_pop(&mut self, close: bool) -> &mut Self {
        self.end(close).pop()
    }

    /// Builds a path object, consuming the builder.
    #[inline]
    pub fn build<P>(self) -> P
    where
        T: Build<PathType = P>,
    {
        self.builder.build()
    }

    /// Adds a transformed line segment to the current sub-path.
    ///
    /// A sub-path must be in progress when this method is called.
    pub fn line_to(&mut self, point: Vec2) -> &mut Self {
        self.builder
            .line_to(vec2p(self.transform.transform_point2(point)));
        self
    }

    /// Adds a transformed quadratic bézier curve to the current sub-path.
    ///
    /// A sub-path must be in progress when this method is called.
    pub fn quadratic_bezier_to(&mut self, ctrl: Vec2, to: Vec2) -> &mut Self {
        self.builder.quadratic_bezier_to(
            vec2p(self.transform.transform_point2(ctrl)),
            vec2p(self.transform.transform_point2(to)),
        );
        self
    }

    /// Adds a transformed cubic bézier curve to the current sub-path.
    ///
    /// A sub-path must be in progress when this method is called.
    pub fn cubic_bezier_to(&mut self, ctrl1: Vec2, ctrl2: Vec2, to: Vec2) -> &mut Self {
        self.builder.cubic_bezier_to(
            vec2p(self.transform.transform_point2(ctrl1)),
            vec2p(self.transform.transform_point2(ctrl2)),
            vec2p(self.transform.transform_point2(to)),
        );
        self
    }

    /// Adds a transformed sub-path containing a rectangle.
    ///
    /// There must be no sub-path in progress when this method is called.
    /// No sub-path is in progress after the method is called.
    pub fn add_rectangle(&mut self, rect: bevy::math::Rect, winding: Winding) -> &mut Self {
        let p0 = vec2p(self.transform.transform_point2(rect.min));
        let p1 = vec2p(self.transform.transform_point2(rect.max));
        self.builder
            .add_rectangle(&lyon::math::Box2D::new(p0, p1), winding);
        self
    }

    /// Adds a transformed sub-path containing a circle.
    ///
    /// There must be no sub-path in progress when this method is called.
    /// No sub-path is in progress after the method is called.
    pub fn add_circle(&mut self, center: Vec2, radius: f32, winding: Winding) -> &mut Self {
        self.builder.add_circle(
            vec2p(self.transform.transform_point2(center)),
            radius,
            winding,
        );
        self
    }

    /// Adds a transformed sub-path containing an ellipse.
    ///
    /// There must be no sub-path in progress when this method is called.
    /// No sub-path is in progress after the method is called.
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
