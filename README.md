# CeBrA Efficiency

Easily calculate the full-energy peak (FEP) efficiency of the CeBr3 Array (CeBrA) with this app.  

### Running Locally

Make sure you are using the latest version of stable Rust by running `rustup update`. Rust is very easy to install on any computer. First, you'll need to install the Rust toolchain (compiler, cargo, etc.). Go to the [Rust website](https://www.rust-lang.org/tools/install) and follow the instructions there.

`cargo run --release`

On Linux, you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libgtk-3-dev`

On Fedora Rawhide, you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

### Example

A previous measurment (from the REU in 2023 with 5 CeBr3 detectors) button is located on the top panel when running locally. This button loads the file at etc/REU_2023.yaml.

### Web Application

The application can be run online [here](https://alconley.github.io/cebra_efficiency/). Files can be
saved (downloaded) and re-loaded back in... straight from the web! For an example, download the file in the etc direction (REU_2023.yaml).

### Other

The UI is pretty self explanatory, so I am not going to write a lot about it.

To change the marker shape, color, and line traits, right click on the plot!

## Fitting

I am using the crate [Varpro](https://github.com/geo-ant/varpro) to do single and double exponential fitting. Make sure to give the initial values of the non-linear parameters in the bottom panel.

I calculate the uncertainity bands the same way pythons [lmfit](https://github.com/lmfit/lmfit-py) package does.

## Efficiency Calculation of CeBrA

Before you can calculate the full-energy peak (FEP) efficiency of CeBrA, you need to have a calibrated $\gamma$ source. At FSU, we have a couple of calibrated sources ($^{60}\mathrm{Co}$, $^{152}\mathrm{Eu}$, and $^{133}\mathrm{Ba}$) as of 2024. Each source has a known activity $A_{0}[\mathrm{kBq}=1000*\frac{\mathrm{disintegration}}{\mathrm{seconds}}]$ at some date ($T_{0}$) with a specific half-life ($T_{1/2}=\frac{\mathrm{ln(2)}}{\lambda}\mathrm{[years]*\frac{365.25[days]}{[years]}}$). The app then calculates the activity of the source ($A$) on the day of the measurement ($T$) based on the radioactive decay law.

**Activity of Source (Radioactive Decay Law)**
$$A(T) = A_{0} \mathrm{[kBq]} * \mathrm{Exp}[-\frac{\lambda [\mathrm{days}] }{T-T_{0}[\mathrm{days}]}] $$

Now that we have the activity of the source on the day of the measurement, we need to find the $\gamma$ lines in the source. For a $^{60}\mathrm{Co}$ source, these would be the 1173.2 keV and 1332.5 keV $\gamma$ rays emitted after the decay of $^{60}\mathrm{Co}$ to $^{60}\mathrm{Ni}$. Each $\gamma$ has a certain intensity ($I_{\gamma}$), which can be found on [NNDC](https://www.nndc.bnl.gov/nudat3/) or elsewhere on the internet. The intensity values for the $\gamma$ rays emitted from a $^{60}\mathrm{Co}$ source are $I_{1173.2}$=99.85(3) and $I_{1332.5}$=99.9826(6) ([60Co decay info](https://www.nndc.bnl.gov/nudat3/decaysearchdirect.jsp?nuc=60Co&unc=NDS)). Our job is to figure out the efficiency, aka how many $\gamma$ rays did we detect ($N_{\gamma}^{detected}$) divided by how many $\gamma$ rays were emitted ($N_{\gamma}^{total}$). To calculate the number of $\gamma$ rays emitted, we need to know the intensity of the line ($I_{\gamma}$), the measurement run time ($T_{measurement}[\mathrm{hours}]$), and the source activity on the day of the measurement ($A(T_{measurement})\mathrm{[kBq]}$).

**Number of $\gamma$'s Emitted (with unit conversion)**
$$N_{\gamma}^{total} = I_{\gamma} * T_{measurement}[\mathrm{hours}] * \frac{3600  \mathrm{[seconds]}}{1 \mathrm{[hours]}} * A(T_{measurement})  \mathrm{[kBq] \frac{1000[Bq]}{[kBq]}  \frac{Counts/[seconds]}{[Bq]} }$$

The number of counts detected will then correspond to a Gaussian peak fitted onto the peak of interest ($N_{\gamma}^{detected}$). Make sure that you take into account background subtraction.

**Efficiency**
$$E_{\gamma} [\\%]= \frac{N_{\gamma}^{detected}}{N_{\gamma}^{total}}*100\\%$$