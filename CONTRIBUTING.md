# Contributing

Contributions to `bwavfile` are welcome!

This project is currently in a maintenance phase, new features are entertained if we
believe there is some demand for them but for the most part our current priorities 
are:

* Addressing bugs.
* Keeping the codebase modern as the Rust language evolves.

This being said, there are some features that we've been wanting for some time and 
new contributors are welcome to take a swing at these:

* Reading `levl` metadata for the generation of waveform overviews.
* Reading `smpl` metadata for reading sampler data, note assignments, loops etc.

## Adding New Features

If you are adding a large amount of new functionality, please weigh the amount of 
work you're doing against the potential benefit, the burden on the maintainers to
review the work, and the technical debt incurred.

In general,

* new features should address new technologies; do not expend large amounts of
  effort to implement features that are primarily of historical interest.
* new features should address the needs of professional users in a music or media
  production environment.

Features that implement new reading functionality must, when submitted, include 
test WAV files created empirically by third-party software. 

## Regarding use of Agents

`bwavfile` is an open-source project that is offered free for no commerical gain, and 
is developed and maintained for educational and creative reasons.

If you use an agent or LLM to produce code for it you are missing out on the benefits 
of contributing to an open-source project, particularly community, collaboration with 
other developers and designers, and being able to learn and experiment without the 
burden of deadlines or worrying about business cases or profits.

This project is supposed to be fun, do not let machines have fun for you.

We can't prevent you from using LLMs to contribute to this project but we ask you 
abide by the following eitiquette when doing so:

* All communication with the maintainers must be written by a human in their own
  voice. Never use an LLM to craft thread comments, discussion posts, issues, emails
  or other correspondence with other developers or the maintainers. 
* PRs must be submitted by a person. Do not allow an agent to submit its own PRs to
  this project.
* Especially if you are a new contributor to this project, please submit only one PR
  at a time and please restrict the subject matter of the PR to a specific unit,
  module or tool. All submissions have to be reviewed and understood by the
  maintainers before they can be merged.

Obviously we can't verify if you follow all of these rules but certain telltale 
traits of LLM-predicted text or code will raise a flag: lack of brevity in 
descriptions or code comments, large amounts of text describing your process or 
steps that add little to understanding the changes you've made, use of an 
obsequious tone or being excessively accomodating, immediately doing requests 
without further discussion or clarifications.
