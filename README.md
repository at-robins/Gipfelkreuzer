<img src="/resources/icon.svg" alt="Gipfelkreuzer icon" width=35% />
<br />
Creates consensus peaks from raw peaks called on ChIP- or ATAC-Seq data.

# Input data

The application requires [GA4GH BED v1.0](https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf) complient BED3+ files as input
containing the genomic locations of all peaks that should be used for consensus peak generation.
Typically this will be multiple files containing the different peaks called on individual samples of a respective experiment.
At minimum the first 3 BED columns (chromosome, start and end coordinate) are required,
but the algorithm was explicitly designed to use the summit information provided for example by MACS3 called narrow peaks.
Summit information is expected at column 10 of the input BED file as offset from the start coordinate as defined
in the [narrowPeak format](https://genome.ucsc.edu/FAQ/FAQformat.html#format12).
If no summit information is present the mean position is used as summit approximation.

# Installation

The application can be downloaded from the [release page](https://github.com/at-robins/Gipfelkreuzer/releases).
Automatically generated installation instructions are also available there.

Alternatively, the application can be built from source when a Rust toolchain is installed:

```bash
# Builds the application from the source directory.
cargo build --release
```

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

# Consensus peak generation algorithms

## Gipfelkreuzer peak merging

The default algorithm. Merges peaks based on summit proximity using their width as a proximity estimator. The algorithm specific `--max-merge-iterations` argument has a reasonable default and should only be changed if you know exactly what you are doing.

```bash
# For example:
Gipfelkreuzer -a harmonised -m 20 -n 2 -o harmonised_consensus_peaks.bed called_peaks_sample_1.narrowPeak called_peaks_sample_2.narrowPeak
```

| Argument (long)        | Argument (short) | Description                                                          |
| ---------------------- | ---------------- | -------------------------------------------------------------------- |
| --max-merge-iterations | -m               | The maximum number of iterative merges for consensus peak generation |

## Simple peak merging

Merges overlapping and directly adjacent peaks using only the start and end coordinates of the peaks.

```bash
# For example:
Gipfelkreuzer -a simple -n 2 -o harmonised_consensus_peaks.bed called_peaks_sample_1.narrowPeak called_peaks_sample_2.narrowPeak
```

## Harmonised peak merging

Implementation according to [Cherchame et al. in 2025](https://www.protocols.io/view/atac-seq-methods-for-consensus-peak-generation-to-36wgq326olk5/v1).
Merges peaks based on summit proximity using a fixed summit distance. Peak shortening for identical peaks was not implemented as it seemed to rely on
undocumented software behaviour and the specification was unclear on cases with multiple identical peaks. The algorithm specific `--harmonising-distance` argument defaults to the distance specified by the implementation reference but can be set as seen fit.

```bash
# For example:
Gipfelkreuzer -a harmonised -d 250 -n 2 -o harmonised_consensus_peaks.bed called_peaks_sample_1.narrowPeak called_peaks_sample_2.narrowPeak
```

| Argument (long)        | Argument (short) | Description                                                              |
| ---------------------- | ---------------- | ------------------------------------------------------------------------ |
| --harmonising-distance | -d               | The maximum distance between summits to merge them into a consensus peak |

# Non algorithm specifc optional command line arguments

For more details run:

```bash
Gipfelkreuzer --help
```

| Argument (long)           | Argument (short) | Description                                                                                               |
| ------------------------- | ---------------- | --------------------------------------------------------------------------------------------------------- |
| --output-file             | -o               | The output file path                                                                                      |
| --bed-output-columns      | -b               | The number of columns to output per consensus peak                                                        |
| --algorithm               | -a               | The algorithm to use for consensus peak generation                                                        |
| --min-peaks-per-consensus | -n               | The minimum number of incorporated raw peaks needed to consider a consensus peak as valid or reproducible |
| --log-level               | -l               | The log level to print while running the application                                                      |

# Cite

Schenk, R. P., & Wiedemann, G. M. (2026). Gipfelkreuzer: Automated consensus peak generation (1.0.1). [https://github.com/at-robins/Gipfelkreuzer](https://github.com/at-robins/Gipfelkreuzer)
