%global debug_package %{nil}

Name:           i3rustus
Version:        0.1.0
Release:        %autorelease
Summary:        i3 status bar implemented in rust
License:        MIT
URL:            https://github.com/blinxen/i3rustus
Source:         https://github.com/blinxen/i3rustus/archive/refs/tags/%{version}.tar.gz

BuildRequires:  rust-packaging >= 21

%description
i3status is my own implementation of i3status.
The goal is that I replace i3status with this and build
custom tools that are not available in i3status.

This project is not intended for other people to use.
It's one of my rust learning projects,
where I am getting to know rust better.

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}

%prep
%autosetup -n %{name}-%{version_no_tilde} -p1
%cargo_prep

%generate_buildrequires
%cargo_generate_buildrequires

%build
%cargo_build

%install
%cargo_install

%if %{with check}
%check
%cargo_test
%endif

%changelog
* Thu Dec 15 2022 blinxen <h-k-81@hotmail.com> - 0.1.0
- Initial spec
