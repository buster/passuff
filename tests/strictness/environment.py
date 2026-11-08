def before_scenario(context, scenario):
    if "skip_probe" in scenario.tags:
        scenario.skip("strictness probe")
