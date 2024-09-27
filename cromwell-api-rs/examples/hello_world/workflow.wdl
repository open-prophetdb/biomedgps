# Example workflow
# Declare WDL version 1.0 if working in Terra
version 1.0
workflow hello_world {
    input {
        String greeting
    }

    call hello_world_task {
        input: greeting = greeting
    }
}

task hello_world_task {
    input {
        String greeting
    }

    command <<<
        echo "${greeting}" > out.tsv

        cat <<EOF > metadata.json
            {
                "files": [
                    {
                        "filepath": "out.tsv",
                        "filetype": "text/tab-separated-values"
                    }
                ],
                "charts": [
                    {
                        "filepath": "output.json",
                        "filetype": "application/json"
                    }
                ]
            }
        EOF
    >>>

    output {
        File out = "out.tsv"
        File metadata = "metadata.json"
    }
}
