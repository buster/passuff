from behave import given


@given("a strictness step that runs")
def step_runs(context):
    context.strictness_ran = True
