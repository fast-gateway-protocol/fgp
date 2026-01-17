import AppKit
import SwiftUI

struct MarketplaceView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                header
                categoryFilter
                content
            }
            .padding(24)
        }
        .frame(minWidth: 800, minHeight: 600)
        .onAppear {
            Task { await appState.registryStore.fetchRegistry() }
        }
    }

    private var header: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text("Marketplace")
                    .font(.largeTitle.bold())
                Text("Discover and install FGP daemons")
                    .foregroundColor(.secondary)
            }
            Spacer()
            Button {
                Task { await appState.registryStore.fetchRegistry() }
            } label: {
                Image(systemName: "arrow.triangle.2.circlepath")
            }
            .buttonStyle(.borderless)
        }
    }

    private var categoryFilter: some View {
        Group {
            if let registry = appState.registryStore.registry {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        categoryButton(title: "All", id: nil)
                        ForEach(registry.categories) { category in
                            categoryButton(title: category.name, id: category.id)
                        }
                    }
                    .padding(.vertical, 4)
                }
            }
        }
    }

    private var content: some View {
        Group {
            if appState.registryStore.loading && appState.registryStore.registry == nil {
                ProgressView()
                    .frame(maxWidth: .infinity, minHeight: 200)
            } else if let error = appState.registryStore.error {
                VStack(alignment: .leading, spacing: 8) {
                    Text(error)
                        .foregroundColor(.red)
                    Button("Try again") {
                        Task { await appState.registryStore.fetchRegistry() }
                    }
                }
                .padding(12)
                .background(RoundedRectangle(cornerRadius: 10).fill(Color.red.opacity(0.1)))
            } else if let registry = appState.registryStore.registry {
                VStack(alignment: .leading, spacing: 20) {
                    if appState.registryStore.selectedCategory == nil && !appState.registryStore.featuredPackages.isEmpty {
                        featuredSection
                    }

                    Text(sectionTitle(for: registry))
                        .font(.title3.bold())
                    VStack(spacing: 10) {
                        ForEach(appState.registryStore.filteredPackages) { pkg in
                            packageRow(pkg)
                        }
                    }
                }
            }
        }
    }

    private var featuredSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Featured")
                .font(.title3.bold())
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                ForEach(appState.registryStore.featuredPackages) { pkg in
                    featuredCard(pkg)
                }
            }
        }
    }

    private func featuredCard(_ pkg: RegistryPackage) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 10) {
                Image(systemName: iconName(pkg.icon))
                    .font(.title2)
                    .foregroundColor(.accentColor)
                VStack(alignment: .leading, spacing: 2) {
                    HStack {
                        Text(pkg.displayName)
                            .font(.headline)
                        if pkg.official {
                            Text("Official")
                                .font(.caption2)
                                .foregroundColor(.accentColor)
                        }
                    }
                    Text(pkg.description)
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .lineLimit(2)
                }
                Spacer()
            }

            HStack {
                Text("\(pkg.methodsCount) methods")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                if pkg.installed {
                    Text("Installed")
                        .font(.caption2)
                        .foregroundColor(.green)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Capsule().fill(Color.green.opacity(0.2)))
                } else {
                    Button("Install") {
                        Task { await appState.registryStore.installPackage(name: pkg.name) }
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(appState.registryStore.installingPackage != nil)
                }
            }
        }
        .padding(12)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color(NSColor.windowBackgroundColor)))
    }

    private func packageRow(_ pkg: RegistryPackage) -> some View {
        HStack(spacing: 12) {
            Image(systemName: iconName(pkg.icon))
                .foregroundColor(.secondary)
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 8) {
                    Text(pkg.displayName)
                        .font(.headline)
                    if pkg.official {
                        Text("Official")
                            .font(.caption2)
                            .foregroundColor(.accentColor)
                    }
                    if pkg.installed && pkg.updateAvailable {
                        Text("Update")
                            .font(.caption2)
                            .foregroundColor(.orange)
                    }
                }
                Text(pkg.description)
                    .font(.caption)
                    .foregroundColor(.secondary)
                HStack(spacing: 12) {
                    Text("v\(pkg.version)")
                    Text("\(pkg.methodsCount) methods")
                    Text(pkg.author)
                }
                .font(.caption2)
                .foregroundColor(.secondary)
            }
            Spacer()
            packageActions(pkg)
        }
        .padding(12)
        .background(RoundedRectangle(cornerRadius: 10).fill(Color(NSColor.windowBackgroundColor)))
    }

    private func packageActions(_ pkg: RegistryPackage) -> some View {
        Group {
            if appState.registryStore.installingPackage == pkg.name {
                HStack(spacing: 8) {
                    ProgressView()
                    Text(appState.registryStore.installProgress?.step ?? "Installing...")
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                }
            } else if pkg.installed {
                HStack(spacing: 8) {
                    Button {
                        if let url = URL(string: pkg.repository) {
                            NSWorkspace.shared.open(url)
                        }
                    } label: {
                        Image(systemName: "link")
                    }
                    .buttonStyle(.borderless)
                    Button("Uninstall") {
                        Task { await appState.registryStore.uninstallPackage(name: pkg.name) }
                    }
                    .buttonStyle(.borderless)
                    .foregroundColor(.red)
                }
            } else {
                Button("Install") {
                    Task { await appState.registryStore.installPackage(name: pkg.name) }
                }
                .buttonStyle(.borderedProminent)
                .disabled(appState.registryStore.installingPackage != nil)
            }
        }
    }

    private func categoryButton(title: String, id: String?) -> some View {
        Button {
            appState.registryStore.selectedCategory = id
        } label: {
            Text(title)
                .font(.caption)
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(
                    Capsule().fill(appState.registryStore.selectedCategory == id ? Color.accentColor : Color(NSColor.controlBackgroundColor))
                )
                .foregroundColor(appState.registryStore.selectedCategory == id ? .white : .primary)
        }
        .buttonStyle(.plain)
    }

    private func iconName(_ icon: String) -> String {
        switch icon {
        case "mail":
            return "envelope"
        case "calendar":
            return "calendar"
        case "code":
            return "chevron.left.slash.chevron.right"
        case "cloud":
            return "cloud"
        case "database":
            return "database"
        case "triangle":
            return "triangle"
        default:
            return "globe"
        }
    }

    private func sectionTitle(for registry: Registry) -> String {
        if let selected = appState.registryStore.selectedCategory {
            return registry.categories.first(where: { $0.id == selected })?.name ?? "Packages"
        }
        return "All Packages"
    }
}
