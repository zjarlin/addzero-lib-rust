use std::fs;
use std::path::{Path, PathBuf};

use az_algorithm::components::face_recognition::assist::compare_face_images_with_output;
use az_algorithm::components::face_recognition::model::ALGORITHM_CODE;

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

const FACE_RECOGNITION_MATERIALS: &[&str] = &[
    "/Users/zjarlin/Desktop/人1.png",
    "/Users/zjarlin/Desktop/人2.png",
    "/Users/zjarlin/Desktop/人3.png",
    "/Users/zjarlin/Desktop/人4.png",
    "/Users/zjarlin/Desktop/人5.png",
    "/Users/zjarlin/Desktop/人6.png",
];

const FACE_RECOGNITION_CASES: &[(usize, usize, &str, bool)] = &[
    (0, 1, "person_01_vs_02", true),
    (2, 3, "person_03_vs_04", true),
    (4, 5, "person_05_vs_06", true),
    (0, 2, "person_01_vs_03", false),
    (0, 3, "person_01_vs_04", false),
    (0, 4, "person_01_vs_05", false),
    (0, 5, "person_01_vs_06", false),
    (1, 2, "person_02_vs_03", false),
    (1, 3, "person_02_vs_04", false),
    (1, 4, "person_02_vs_05", false),
    (1, 5, "person_02_vs_06", false),
    (2, 4, "person_03_vs_05", false),
    (2, 5, "person_03_vs_06", false),
    (3, 4, "person_04_vs_05", false),
    (3, 5, "person_04_vs_06", false),
];

fn material_path(index: usize) -> PathBuf {
    std::fs::canonicalize(FACE_RECOGNITION_MATERIALS[index])
        .expect("用户提供的人脸比对素材必须存在")
}

fn output_dir() -> PathBuf {
    workspace_root()
        .join("target/az-algorithm-results")
        .join("face_recognition")
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

#[test]
fn face_recognition_should_compare_user_people_materials_and_write_similarity() -> anyhow::Result<()>
{
    // 用户提供的人脸比对素材：
    // /Users/zjarlin/Desktop/人1.png ... /Users/zjarlin/Desktop/人6.png
    //
    // 模型：
    // crates/algorithm/algorithm/resources/face_recognition/models/face_recognition_sface_2021dec.onnx
    //
    // 输出：
    // target/az-algorithm-results/face_recognition/user_people_materials/person_01_vs_02/similarity.json
    // target/az-algorithm-results/face_recognition/user_people_materials/person_03_vs_04/similarity.json
    // target/az-algorithm-results/face_recognition/user_people_materials/person_05_vs_06/comparison.png
    assert_eq!(ALGORITHM_CODE, "face_recognition");

    let output_root = output_dir().join("user_people_materials");
    if output_root.exists() {
        fs::remove_dir_all(&output_root)?;
    }
    let mut mismatches = Vec::new();
    for (pair_index, (left_index, right_index, output_name, expected_same_identity)) in
        FACE_RECOGNITION_CASES.iter().copied().enumerate()
    {
        let left = material_path(left_index);
        let right = material_path(right_index);
        let result = compare_face_images_with_output(&left, &right, output_root.join(output_name))?;

        // 关键断言：face_recognition 必须比较两路 embedding，并写出该 pair 的相似度。
        assert_eq!(result.algorithm_code, ALGORITHM_CODE);
        assert!(result.probe.embedding_dimension > 0);
        assert_eq!(
            result.probe.embedding_dimension,
            result.reference.embedding_dimension,
            "第 {} 组人脸比对 embedding 维度必须一致",
            pair_index + 1
        );
        assert!(
            (-1.0..=1.0).contains(&result.cosine_similarity),
            "第 {} 组 face_recognition 余弦相似度必须在 -1..=1，实际为 {}",
            pair_index + 1,
            result.cosine_similarity
        );
        eprintln!(
            "{} same_identity={} cosine_similarity={:.6} threshold={:.6}",
            output_name,
            result.same_identity,
            result.cosine_similarity,
            result.same_identity_threshold
        );
        if result.same_identity != expected_same_identity {
            mismatches.push(format!(
                "{} expected={} actual={} cosine={:.6} threshold={:.6}",
                output_name,
                expected_same_identity,
                result.same_identity,
                result.cosine_similarity,
                result.same_identity_threshold
            ));
        }
        assert!(
            result.same_identity_threshold > 0.0 && result.same_identity_threshold < 1.0,
            "face_recognition 阈值必须是 0..1 内的可用分界"
        );
        assert_eq!(
            result.probe.detected_face_count, 1,
            "{} 的 probe 输入应检测到 1 张主脸",
            output_name
        );
        assert_eq!(
            result.reference.detected_face_count, 1,
            "{} 的 reference 输入应检测到 1 张主脸",
            output_name
        );
        assert_face_recognition_outputs_exist(&result);
    }
    assert!(
        mismatches.is_empty(),
        "face_recognition 同人判断不符合用户标注：\n{}",
        mismatches.join("\n")
    );
    Ok(())
}

fn assert_face_recognition_outputs_exist(
    result: &az_algorithm::components::face_recognition::model::FaceRecognitionRun,
) {
    assert_existing_file(&result.files.probe.source_input);
    assert_existing_file(&result.files.probe.model_input_preview);
    assert_existing_file(&result.files.probe.raw_outputs_json);
    assert_existing_file(&result.files.probe.raw_output_review);
    assert_existing_file(&result.files.reference.source_input);
    assert_existing_file(&result.files.reference.model_input_preview);
    assert_existing_file(&result.files.reference.raw_outputs_json);
    assert_existing_file(&result.files.reference.raw_output_review);
    assert_existing_file(&result.files.similarity_json);
    assert_existing_file(&result.files.comparison_image);
}
