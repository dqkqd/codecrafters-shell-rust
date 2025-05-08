use rustyline::{
    completion::{Completer, Pair},
    highlight::Highlighter,
    Completer, Context, Helper, Hinter, Validator,
};

#[derive(Helper, Completer, Hinter, Validator)]
pub(crate) struct ShellHelper {
    #[rustyline(Completer)]
    pub completer: ShellCompleter,
}

impl Highlighter for ShellHelper {}

pub(crate) struct ShellCompleter {}
impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let _ = (line, pos, _ctx);
        Ok((0, Vec::with_capacity(0)))
    }
}
