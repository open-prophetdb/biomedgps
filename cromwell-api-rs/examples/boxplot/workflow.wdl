# Declare WDL version 1.0
version 1.0

workflow boxplot {
    call boxplot_task
}

task boxplot_task {
    input {
        String exp_file
        String sample_info_file
        Array[String] which_ids
        Array[String] which_gene_symbols
        Array[String] which_groups
        String method
        String log_scale
        String enable_label
        String enable_log2fc
        String script_dir
    }

    command <<<
        echo "Generating a args file, named args.json"
        echo "exp_file: ~{exp_file}"
        echo "sample_info_file: ~{sample_info_file}"
        echo "which_ids: ~{sep=", " which_ids}"
        echo "which_gene_symbols: ~{sep=", " which_gene_symbols}"
        echo "which_groups: ~{sep=", " which_groups}"
        echo "method: ~{method}"
        echo "log_scale: ~{log_scale}"
        echo "enable_label: ~{enable_label}"
        echo "enable_log2fc: ~{enable_log2fc}"
        echo "script_dir: ~{script_dir}"

        cat <<EOF > args.json
        {
            "exp_file": "~{exp_file}",
            "sample_info_file": "~{sample_info_file}",
            "which_ids": "~{sep=',' which_ids}",
            "which_gene_symbols": "~{sep=',' which_gene_symbols}",
            "which_groups": "~{sep=',' which_groups}",
            "method": "~{method}",
            "log_scale": "~{log_scale}",
            "enable_label": "~{enable_label}",
            "enable_log2fc": "~{enable_log2fc}",
            "output_file": "output.json"
        }
        EOF

        echo "Generating a metadata file, named metadata.json"
        cat <<EOF > metadata.json
        {
            "files": [
                {
                    "filename": "~{basename(exp_file)}",
                    "filetype": "text/tab-separated-values"
                },
                {
                    "filename": "~{basename(sample_info_file)}",
                    "filetype": "text/tab-separated-values"
                }
            ],
            "charts": [
                {
                    "filename": "output.json",
                    "filetype": "application/json"
                }
            ]
        }
        EOF

        Rscript ~{script_dir}/boxplot.R args.json

        cp ~{exp_file} ./
        cp ~{sample_info_file} ./
    >>>

    output {
        File metadata = "metadata.json"
        File out_plot = "output.json"
        File exp_file = "~{basename(exp_file)}"
        File sample_info_file = "~{basename(sample_info_file)}"
    }
}
