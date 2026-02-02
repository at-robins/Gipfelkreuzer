<img src="/resources/icon.svg" alt="Gipfelkreuzer icon" width=35% />
<br />
Creates consensus peaks from raw peaks called on ChIP- or ATAC-Seq data.

# Input data

The tool requires [GA4GH BED v1.0](https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf) complient BED3+ files as input
containing the genomic locations of all peaks that should be used for consensus peak generation.
Typically this will be multiple files containing the different peaks called on individual samples of a respective experiment.
At minimum the first 3 BED columns (chromosome, start and end coordinate) are required,
but the algorithm was explicitly designed to use the summit information provided for example by MACS3 called narrow peaks.
Summit information is expected at column 10 of the input BED file as offset from the start coordinate as defined
in the [narrowPeak format](https://genome.ucsc.edu/FAQ/FAQformat.html#format12).
If no summit information is present the mean position is used as summit approximation.

# Run

You can either directly use the executable:

```bash
Gipfelkreuzer -o <output_file> <input_file_1> <input_file_2> <...> <input_file_n>
# For example:
Gipfelkreuzer -o consensus_peaks.bed called_peaks_sample_1.narrowPeak called_peaks_sample_2.narrowPeak called_peaks_sample_3.narrowPeak
```

Or use the provided `Dockerfile`. First you need to build the Docker image.
This only needs to be done once:

```bash
# Builds the docker image.
cd path/to/Gipfelkreuzer/
docker build -t "gipfelkreuzer:latest" .
```

Then you can run a docker container. You need to bind an input and output directory
containg your peak files (in this example the directory `io` in the current working directory):

```bash
# Runs a docker container.
docker run --rm --name gipfelkreuzer --mount type=bind,source=./io,target=/io gipfelkreuzer:latest -o /io/consensus_peaks.bed /io/called_peaks_sample_1.narrowPeak /io/called_peaks_sample_2.narrowPeak
```

# Optional command line arguments

The default values of algorithm specific arguments are optimised for a conventional run and should only be changed when you know exactly what you are doing. For more details run:

```bash
Gipfelkreuzer --help
```

| Argument (long)        | Argument (short) | Description                                                                                                 |
| ---------------------- | ---------------- | ----------------------------------------------------------------------------------------------------------- |
| --output-file          | -o               | The output file path                                                                                        |
| --bed-output-columns   | -b               | The number of columns to output per consensus peak                                                          |
| --algorithm            | -a               | The algorithm to use for consensus peak generation                                                          |
| --max-merge-iterations | -m               | The maximum number of iterative merges for consensus peak generation when using the Gipfelkreuzer algorithm |
| --log-level            | -l               | The log level to print while running the tool                                                               |

# Cite

Schenk, R. P., & Wiedemann, G. M. (2026). Gipfelkreuzer: Automated consensus peak generation (0.1). [https://github.com/at-robins/Gipfelkreuzer](https://github.com/at-robins/Gipfelkreuzer)
