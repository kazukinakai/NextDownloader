import Foundation

struct VideoInfo: Codable {
    let id: String
    let title: String
    let description: String?
    let thumbnail: String?
    let duration: Double?
    let uploadDate: String?
    let uploader: String?
    let formats: [VideoFormat]?
    
    enum CodingKeys: String, CodingKey {
        case id
        case title
        case description
        case thumbnail
        case duration
        case uploadDate = "upload_date"
        case uploader
        case formats
    }
}

struct VideoFormat: Codable {
    let formatId: String
    let url: String
    let ext: String
    let filesize: Int?
    let width: Int?
    let height: Int?
    
    enum CodingKeys: String, CodingKey {
        case formatId = "format_id"
        case url
        case ext
        case filesize
        case width
        case height
    }
}
