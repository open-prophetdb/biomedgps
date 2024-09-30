# Boxplot

Generate boxplot for selected genes / proteins across groups.

## Arguments

- `exp_file`: Expression matrix file.
- `sample_info_file`: Sample information file.
- `which_entrez_ids`: Which entrez ids to select.
- `which_gene_symbols`: Which gene symbols to select.
- `which_groups`: Which groups to select.
- `output_file`: Output file.
- `method`: Method to use for boxplot. Supported methods: `t.test`, `wilcox.test`, `anova`, `kruskal.test`. Default: `t.test`.
- `log_scale`: Whether to use log scale for boxplot. TRUE or FALSE. Default: FALSE.
- `enable_label`: Whether to enable label for boxplot. TRUE or FALSE. Default: FALSE.
- `enable_log2fc`: Whether to enable log2 fold change for boxplot. TRUE or FALSE. Default: FALSE.
