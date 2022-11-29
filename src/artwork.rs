use crate::{App, BaseModel, Key};

/// The options that can be set when creating an artwork.
pub struct Options {
    /// Chromatic aberration of the animation.
    pub chroma: f32,
    /// Number of sample per frame in the motion blur.
    pub sample_per_frame: i32,
    /// Shutter angle. Defines how far the frames of the motion blur will be
    /// selected.
    pub shutter_angle: f64,
    /// Provide an extra
    pub extra_tex: Option<Vec<String>>,
    pub noise_amount: f32,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            chroma: 0.,
            sample_per_frame: 1,
            shutter_angle: 0.,
            extra_tex: None,
            noise_amount: 0.,
        }
    }
}

/// The Artwork trait defines your animation.
///
/// The easiest way to get started implementing an artwork is to copy the
/// template.rs in the examples/ folder of this crate.
pub trait Artwork {
    /// This function creates a new instance of the artwork. It should define
    /// all the attributes that the artwork will use.
    fn new(base: BaseModel) -> Self;
    /// This is the main drawing function in the artwork. It should be
    /// deterministic as a function of `time` to ensure good results if the
    /// number of sample per frame is above 1.
    fn draw_at_time(&mut self, time: f64);
    fn get_model(&self) -> &BaseModel;
    fn get_mut_model(&mut self) -> &mut BaseModel;
    /// You should implement this function to define a custom animation length.
    /// For example
    ///
    /// ```ignore
    /// fn n_sec(&self) -> Option<u32> {
    ///     Some(15)
    /// }
    /// ```
    fn n_sec(&self) -> Option<u32> {
        None
    }
    /// You should implement this function if you want to set some of the
    /// parameters available in [Options]. For example:
    /// ```
    /// # use artworks::Options;
    /// fn get_options() -> Option<Options> {
    ///     Some(Options {
    ///         chroma: 0.5,
    ///         sample_per_frame: 1,
    ///         shutter_angle: 0.1,
    ///         extra_tex: None,
    ///         noise_amount: 0.2
    ///     })
    /// }
    /// ```
    fn get_options() -> Option<Options> {
        None
    }
    /// Define what happens when you press a key, useful when you want to reset
    /// some state when starting to record, or cycle through color palette when
    /// pressing P for example.
    fn key_pressed(&mut self, _app: &App, _key: Key) {}
}
