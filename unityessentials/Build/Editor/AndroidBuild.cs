using System;
using System.IO;
using UnityEditor;
using UnityEditor.Build.Reporting;
using UnityEngine;

namespace Builds
{
    public static class AndroidBuild
    {
        static string GetBuildName()
        {
            var version = Application.version.Split('.');
            version[^1] = (int.Parse(version[^1]) + 1).ToString();
            return
                $"{Application.productName.Replace(" ", "")}.{string.Join(".", version)}.{DateTime.Now:dd.M.yyyy}.apk";
        }

        public static UnityOutput AndroidDevelopment(BuildParams buildParams)
        {
            EditorUserBuildSettings.development = true;
            return BuildCore(buildParams);
        }

        public static UnityOutput AndroidRelease(BuildParams buildParams)
        {
            return BuildCore(buildParams);
        }

        static UnityOutput BuildCore(BuildParams buildParams)
        {
            PlayerSettings.SetScriptingBackend(BuildTargetGroup.Android, ScriptingImplementation.IL2CPP);
            EditorUserBuildSettings.SwitchActiveBuildTarget(BuildTargetGroup.Android, BuildTarget.Android);
            var buildName = GetBuildName();
            BuildReport report = BuildPipeline.BuildPlayer(buildParams.Scenes,
                Path.Combine(buildParams.BuildPath, buildName),
                BuildTarget.Android,
                BuildOptions.None);
            int code = (report.summary.result == BuildResult.Succeeded) ? 0 : 1;
            return new UnityOutput
            {
                build_path = Path.Combine(buildParams.BuildPathLocal, buildName).Replace('\\', '/'),
                platform = buildParams.Settings.platform,
                exit_code = code,
            };
        }
    }
}