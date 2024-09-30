# Arguments
args <- commandArgs(trailingOnly = TRUE)
args_json_file <- args[1]

# Load JSON
args_json <- jsonlite::fromJSON(args_json_file)
log_scale <- as.logical(args_json$log_scale)
enable_label <- as.logical(args_json$enable_label)
enable_log2fc <- as.logical(args_json$enable_log2fc)

# Load OmixInsightR
library(OmixInsightR)

# Run OmixInsightR for boxplot
print("Running OmixInsightR for boxplot")

print("Querying data from expression matrix and sample information")
which_entrez_ids <- unlist(strsplit(args_json$which_entrez_ids, ","))
which_gene_symbols <- unlist(strsplit(args_json$which_gene_symbols, ","))
which_groups <- unlist(strsplit(args_json$which_groups, ","))

print(paste("Querying data from expression matrix and sample information:", args_json$exp_file, args_json$sample_info_file, which_entrez_ids, which_gene_symbols, which_groups))
d <- query_data(
    exp_file = args_json$exp_file,
    sample_info_file = args_json$sample_info_file,
    which_entrez_ids = which_entrez_ids,
    which_gene_symbols = which_gene_symbols,
    which_groups = which_groups
)

print("Generating boxplot and saving to file")
boxplotly(d, output_file = args_json$output_file, method = args_json$method, log_scale = log_scale, enable_label = enable_label, enable_log2fc = enable_log2fc)