//
//  Teams.swift
//  beartracks
//
//  Created by Jayen Agrawal on 1/9/24.
//

import Foundation
import SwiftUI

struct Teams: View {
    @EnvironmentObject var appState: AppState
    @State private var performanceValue: Int = 0
    @State var firstPaint = true
    @State var selectedTeam: Int? = nil
    
    var body: some View {
        VStack {
            NavigationView {
                if !appState.teamsList.isEmpty && !appState.teamsList[0].isEmpty {
                    List {
                        ForEach(Array(appState.teamsList[0].enumerated()), id: \.element.team.team) { index, team in
                            NavigationLink(tag: index, selection: self.$selectedTeam, destination: {
                                TeamView(dataItems: TeamViewModel(team: String(team.team.team)))
                                    .environmentObject(appState)
                            }, label: {
                                VStack {
                                    HStack {
                                        Text("\(String(index + 1))")
#if !os(watchOS)
                                            .font(.title)
#else
                                            .font(.title3)
#endif
                                            .padding(.leading)
                                            .frame(maxWidth: .infinity, alignment: .leading)
                                        Text("\(String(team.team.team))")
#if !os(watchOS)
                                            .font(.title)
#else
                                            .font(.title3)
#endif
                                            .padding(.trailing)
                                            .frame(maxWidth: .infinity, alignment: .trailing)
                                    }
                                    HStack {
                                        ProgressView(
                                            value: max(team.performanceValue(type: performanceValue) ?? 0, 0),
                                            total: max(appState.teamsList[0][0].performanceValue(type: performanceValue) ?? 0, 1)
                                        )
                                        .padding([.leading, .trailing])
                                    }
                                }
#if targetEnvironment(macCatalyst)
                                .padding([.top, .bottom])
#endif
#if os(visionOS)
                                .padding(.bottom)
#endif
                            })
#if os(iOS)
                            .listRowBackground(UIDevice.current.userInterfaceIdiom == .pad ? Color.primary.colorInvert() : nil)
#elseif targetEnvironment(macCatalyst)
                            .listRowBackground(Color.primary.colorInvert())
#endif
                        }
#if os(watchOS)
                        Section {
                            VStack {
                                NavigationLink(destination: SettingsView().environmentObject(appState)) {
                                    HStack {
                                        Text("Settings")
                                        Spacer()
                                        Label("", systemImage: "chevron.forward")
                                            .labelStyle(.iconOnly)
                                    }
                                    .padding([.leading, .trailing])
                                }
                            }
                            .padding([.leading, .trailing])
                        }
#endif
                    }
                    .navigationTitle("Teams")
#if !os(watchOS)
                    .toolbar {
                        ToolbarItem(placement: .topBarTrailing) {
                            Picker(selection: $performanceValue, content: {
                                Label("Standard", systemImage: "line.3.horizontal.decrease.circle")
                                    .tag(0)
                                Label("Intake Speed", systemImage: "tray.and.arrow.down")
                                    .tag(1)
                                Label("Movement Speed", systemImage: "arrow.up.and.down.and.arrow.left.and.right")
                                    .tag(2)
                                Label("Outtake Speed", systemImage: "paperplane")
                                    .tag(3)
                                Label("Cycle Speed", systemImage: "arrow.triangle.2.circlepath")
                                    .tag(4)
                                Label("Defense", systemImage: "shield")
                                    .tag(5)
                            }, label: {
                                Label("type", systemImage: "line.3.horizontal.decrease.circle")
                                    .labelStyle(.iconOnly)
                            })
                            .onChange(of: performanceValue) { _ in
                                appState.teamsList[0].sort {
                                    if let mainWeightA = $0.performanceValue(type: performanceValue), let mainWeightB = $1.performanceValue(type: performanceValue) {
                                        return mainWeightA > mainWeightB
                                    } else {
                                        return true
                                    }
                                }
                            }
                        }
                    }
#endif
                } else {
                    if appState.teamsLoadStatus.1 {
                        VStack {
                            Label("failed", systemImage: "xmark.seal.fill")
                                .padding(.bottom)
                                .labelStyle(.iconOnly)
                                .foregroundStyle(Color.pink)
                            Text("load failed")
                                .padding(.bottom)
                        }
                        .navigationTitle("Teams")
                    } else {
                        if appState.teamsLoadStatus.2 {
                            Form {
                                Label("none", systemImage: "questionmark.app.dashed")
                                    .padding(.bottom)
                                    .labelStyle(.iconOnly)
                                    .foregroundStyle(Color.pink)
                                Text("no data")
                                    .padding(.bottom)
#if os(watchOS)
                                Section {
                                    VStack {
                                        NavigationLink(destination: SettingsView()) {
                                            HStack {
                                                Text("Settings")
                                                Spacer()
                                                Label("", systemImage: "chevron.forward")
                                                    .labelStyle(.iconOnly)
                                            }
                                            .padding([.leading, .trailing])
                                        }
                                    }
                                    .padding([.leading, .trailing])
                                }
#endif
                            }
                            .navigationTitle("Teams")
                        } else {
                            VStack {
                                Label("loading", systemImage: "hourglass")
                                    .padding(.bottom)
                                    .labelStyle(.iconOnly)
                                Text("loading teams...")
                                    .padding(.bottom)
                            }
                            .navigationTitle("Teams")
                        }
                    }
                }
            }
        }
        .refreshable {
            appState.fetchTeamsJson()
        }
        .onAppear {
            if firstPaint {
                appState.fetchTeamsJson()
                firstPaint = false
            }
        }
    }
}

#Preview {
    Teams()
}
