use dioxus::prelude::*;

#[derive(Clone, Debug)]
pub enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}

pub trait Input {
    fn name(&self) -> &'static str;
}

pub trait UIOutput {
    fn render(&self) -> Element;
    fn description(&self) -> &'static str;
}

pub trait MealyMachine<InputType, OutputType> {
    fn step(&mut self, input: InputType) -> OutputType;
    
    fn then<Other, OtherOutput>(mut self, mut other: Other) -> impl MealyMachine<InputType, OtherOutput>
    where
        Self: Sized,
        Other: MealyMachine<OutputType, OtherOutput>,
    {
        move |input| {
            let intermediate = self.step(input);
            other.step(intermediate)
        }
    }
    
    fn and_then<F, NewMachine, NewOutput>(mut self, f: F) -> impl MealyMachine<InputType, NewOutput>
    where
        Self: Sized,
        F: Fn(OutputType) -> NewMachine,
        NewMachine: MealyMachine<InputType, NewOutput>,
        InputType: Clone,
    {
        move |input: InputType| {
            let output = self.step(input.clone());
            let mut new_machine = f(output);
            new_machine.step(input)
        }
    }
    
    fn compose<Other, OtherInput>(mut self, mut other: Other) -> impl MealyMachine<OtherInput, OutputType>
    where
        Self: Sized,
        Other: MealyMachine<OtherInput, InputType>,
    {
        move |input| {
            let intermediate = other.step(input);
            self.step(intermediate)
        }
    }



    fn map<NewOutput, MapFn>(mut self, f: MapFn) -> impl MealyMachine<InputType, NewOutput>
    where
        Self: Sized,
        MapFn: Fn(OutputType) -> NewOutput,
    {
        move |input| f(self.step(input))
    }
    fn dimap<NewInput, NewOutput, InputMapFn, OutputMapFn>(
        mut self, 
        input_f: InputMapFn, 
        output_f: OutputMapFn
    ) -> impl MealyMachine<NewInput, NewOutput>
    where
        Self: Sized,
        InputMapFn: Fn(NewInput) -> InputType,
        OutputMapFn: Fn(OutputType) -> NewOutput,
    {
        move |new_input| output_f(self.step(input_f(new_input)))
    }


    fn first<D>(mut self) -> impl MealyMachine<(InputType, D), (OutputType, D)>
    where
        Self: Sized,
    {
        move |(input, d)| (self.step(input), d)
    }
    
    fn second<D>(mut self) -> impl MealyMachine<(D, InputType), (D, OutputType)>
    where
        Self: Sized,
    {
        move |(d, input)| (d, self.step(input))
    }
    
    fn split<Other, OtherOutput>(mut self, mut other: Other) -> impl MealyMachine<(InputType, InputType), (OutputType, OtherOutput)>
    where
        Self: Sized,
        Other: MealyMachine<InputType, OtherOutput>,
        InputType: Clone,
    {
        move |(input1, input2)| (self.step(input1), other.step(input2))
    }


    fn fanout<Other, OtherOutput>(mut self, mut other: Other) -> impl MealyMachine<InputType, (OutputType, OtherOutput)>
    where
        Self: Sized,
        Other: MealyMachine<InputType, OtherOutput>,
        InputType: Clone,
    {
        move |input : InputType| (self.step(input.clone()), other.step(input))
    }
    
    fn left<D>(mut self) -> impl MealyMachine<Either<InputType, D>, Either<OutputType, D>>
    where
        Self: Sized,
    {
        move |either_input| match either_input {
            Either::Left(input) => Either::Left(self.step(input)),
            Either::Right(d) => Either::Right(d),
        }
    }
    
    fn right<D>(mut self) -> impl MealyMachine<Either<D, InputType>, Either<D, OutputType>>
    where
        Self: Sized,
    {
        move |either_input| match either_input {
            Either::Left(d) => Either::Left(d),
            Either::Right(input) => Either::Right(self.step(input)),
        }
    }
    
    fn choice<Other, OtherInput, OtherOutput>(mut self, mut other: Other) -> impl MealyMachine<Either<InputType, OtherInput>, Either<OutputType, OtherOutput>>
    where
        Self: Sized,
        Other: MealyMachine<OtherInput, OtherOutput>,
    {
        move |either_input| match either_input {
            Either::Left(input) => Either::Left(self.step(input)),
            Either::Right(other_input) => Either::Right(other.step(other_input)),
        }
    }
    
    fn merge<Other, OtherInput>(mut self, mut other: Other) -> impl MealyMachine<Either<InputType, OtherInput>, OutputType>
    where
        Self: Sized,
        Other: MealyMachine<OtherInput, OutputType>,
    {
        move |either_input| match either_input {
            Either::Left(input) => self.step(input),
            Either::Right(other_input) => other.step(other_input),
        }
    }
    
    fn unfirst<D>(mut self) -> impl MealyMachine<InputType, OutputType>
    where
        Self: Sized + MealyMachine<(InputType, D), (OutputType, D)>,
        D: Default,
    {
        move |input| self.step((input, D::default())).0
    }
    
    fn unsecond<D>(mut self) -> impl MealyMachine<InputType, OutputType>
    where
        Self: Sized + MealyMachine<(D, InputType), (D, OutputType)>,
        D: Default,
    {
        move |input| self.step((D::default(), input)).1
    }
    
    fn feedback<D>(mut self) -> impl MealyMachine<InputType, OutputType>
    where
        Self: Sized + MealyMachine<(InputType, D), (OutputType, D)>,
        D: Default,
    {
        move |input| {
            let (output, _feedback) = self.step((input, D::default()));
            output
        }
    }
    
    fn duplicate(self) -> impl MealyMachine<InputType, Self>
    where
        Self: Sized + Clone,
    {
        move |input| {
            let mut machine_copy = self.clone();
            let _output = machine_copy.step(input);
            machine_copy
        }
    }
    
    fn extend<NewOutput, ExtendFn, M>(self, f: ExtendFn) -> impl MealyMachine<InputType, NewOutput>
    where
        Self: Sized + Clone,

        ExtendFn: Fn(Self) -> NewOutput,
    {
        self.duplicate().map(f)
    }
    
    fn combine<Other>(mut self, mut other: Other) -> impl MealyMachine<InputType, OutputType>
    where
        Self: Sized,
        Other: MealyMachine<InputType, OutputType>,
        OutputType: Semigroup,
        InputType: Clone,
    {
        move |input: InputType| self.step(input.clone()).combine(other.step(input))
    }
    
    fn empty() -> impl MealyMachine<InputType, OutputType>
    where
        OutputType: Monoid,
    {
        move |_input: InputType| OutputType::empty()
    }
    
}


impl<Input, Output, F> MealyMachine<Input, Output> for F
where
    F: FnMut(Input) -> Output,
{
    fn step(&mut self, input: Input) -> Output {
        self(input)
    }
}





pub fn unfold_mealy<State, Input, Output, TransitionFn>(
    transition: TransitionFn,
    initial_state: State,
) -> impl MealyMachine<Input, Output>
where
    TransitionFn: Fn(&mut State, Input) -> Output,
{
    let mut state = initial_state;
    move |input| transition(&mut state, input)
}

pub fn log_mealy<A>() -> impl MealyMachine<A, A>
where
    A: Semigroup + Clone,
{
    let mut history: Option<A> = None;
    move |input: A| {
        match &mut history {
            None => {
                history = Some(input.clone());
                input
            }
            Some(h) => {
                let combined = h.clone().combine(input.clone());
                *h = combined.clone();
                combined
            }
        }
    }
}

pub trait Semigroup {
    fn combine(self, other: Self) -> Self;
}

pub trait Monoid: Semigroup {
    fn empty() -> Self;
}
