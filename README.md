<img src="/resources/icon.svg" alt="Gipfelkreuzer icon" width=35% />
\
\
\
Creates consensus peaks from raw peaks called on ChIP- or ATAC-Seq data.

# Input data

The tool requires a [GA4GH BED v1.0](https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf) complient BED3+ file as input
containing the genomic locations of all peaks that should be used for consensus peak generation.
Typically this will be a merged file containing all peaks called on individual samples of a respective experiment.
At minimum the first 3 BED columns (chromosome, start and end coordinate) are required,
but the algorithm was explicitly designed to use the summit information provided for example by MACS3 called narrow peaks.
Summit information is expected at column 10 of the input BED file as offset from the start coordinate as defined
in the [narrowPeak format](https://genome.ucsc.edu/FAQ/FAQformat.html#format12).
If no summit information is present the mean position is used as summit approximation.


# Run

```bash
Gipfelkreuzer -o consensus_peaks.bed called_peaks.narrowPeak
```

# Optional command line arguments

For more details run:

```bash
Gipfelkreuzer --help
```

| Argument (long)        | Argument (short) | Description                                                          |
| ---------------------- | ---------------- | -------------------------------------------------------------------- |
| --output-file          | -o               | The output file path                                                 |
| --bed-output-columns   | -b               | The number of columns to output per consensus peak                   |
| --max-merge-iterations | -m               | The maximum number of iterative merges for consensus peak generation |
| --log-level            | -l               | The log level to print while running the tool                        |

# Cite

Schenk, R. P., & Wiedemann, G. M. (2026). Gipfelkreuzer: Automated consensus peak generation (0.1). [https://github.com/at-robins/espe](https://github.com/at-robins/espe)
