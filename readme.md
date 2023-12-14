A local command line tool for managing and executing LLamafiles [llamafiles](https://github.com/Mozilla-Ocho/llamafile)


# Quick Start
> ./lai test

Downloads the llava model and runs an arbitrary test prompt

# list all available models
> ./lai list

lists all models recognized by local-ai

# list all downloaded models
> ./lai list offline

lists all models currently downloaded and available to run and serve

# explain model
> ./lai {model-name} params

explains the relevant parameters and defines how to pass them to the specified model

# run model
> ./lai {model-name} run {relevant} {model} {parameters}

runs the model using the available parameters that are available to the specified model

# serve model
> ./lai {model-name} serve {relevant} {model} {parameters}

creates a listening service to accept model parameter requests
