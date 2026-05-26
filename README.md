## Exercise

In `src/lib.rs`, the `Orderer` struct implements some logic for buffering and ordering elements that implement the `Orderable` trait.
We want you to optimize this code for performance, modifying the implemention and, if justified, the interface of the service as well.
In this case, understand as performance the maximum supported call rate for `push` assuming `flush` is called every few seconds.
You can also assume the elements will be mostly ordered (but not completely!).

Some directives:
* Justify all of your choices.
* Consider the time spent on allocations.
* Follow software development practices as you would for a regular, professional project (except for part where you ship it as a zip file...).
* The exercise is designed to not need any dependencies, if you need to use one, please justify it.

Note that this is an exercise, and we do not intend you to spend more than a couple of hours on it.
If you get stuck please feel free to take some assumptions/simplify the problem, but state the changes clearly and why you made them.